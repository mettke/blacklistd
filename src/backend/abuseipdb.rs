use crate::{
    args::CliArguments,
    db_op,
    helper::get_elapsed_time,
    middleware::diesel::{DBType, DieselPooledConnection},
    schema::blacklist::dsl::*,
    types::{backend_type::BackendType, blacklist::Blacklist},
};
use diesel::{
    BoolExpressionMethods, ExpressionMethods, QueryDsl,
    RunQueryDsl,
};
use log::{debug, error, info, warn};
use reqwest::{
    header::{HeaderName, ACCEPT},
    Client,
};
use std::any::Any;

macro_rules! abipdb_call {
    ($client:expr, $url:expr, $accept:expr, $key:expr) => {
        {
            let mut response = $client
                .get($url)
                .header(ACCEPT, $accept)
                .header(
                    HeaderName::from_lowercase(b"key")
                        .expect("Invalid header"),
                    $key.api_abuseipdb.as_ref().map(|s| &s[..]).unwrap_or(""),
                )
                .send()
                .map_err(|err| {
                    error!(
                        "Unable to send request to abuseipdb: {}",
                        err
                    );
                })
                .ok()?;
            let code = response.status();
            let mut response_val = None;
            if code.is_success() {
                response_val = response
                        .text()
                        .map_err(|err| {
                            error!(
                                "Unable to parse check response from abuseipdb: {}",
                                err
                            );
                        })
                        .ok();
            } else if code.is_client_error() {
                warn!(
                    "Hit a Request Limit while fetching abuseipdb: {}",
                    code
                );
            } else {
                error!(
                    "Hit a Request Error while fetching abuseipdb: {}",
                    code
                );
            }
            response_val
        }
    }
}

pub fn update_abuseipdb(
    args: &CliArguments,
    db_conn: &dyn Any,
    db_type: DBType,
    client: &Client,
) {
    info!("Fetching abuseipdb");
    let time = time::now();
    if let Some(response) =
        fetch_abuseipdb_blacklist(args, client)
    {
        debug!("Storing abuseipdb");
        store_abuseipdb(db_conn, &response, db_type);
    }
    // store_abuseipdb(
    //     db_conn,
    //     &include_str!("../blacklist"),
    //     db_type,
    // );
    debug!("Updating old abuseipdb ips");
    update_old_ips(args, db_conn, db_type, client);
    info!(
        "abuseipdb completed after {} s",
        get_elapsed_time(time)
    );
}

#[allow(dead_code)]
fn fetch_abuseipdb_blacklist(
    args: &CliArguments,
    client: &Client,
) -> Option<String> {
    abipdb_call!(
        client,
        "https://api.abuseipdb.com/api/v2/blacklist",
        "text/plain",
        args
    )
}

fn store_abuseipdb(
    db_conn: &dyn Any,
    response: &str,
    db_type: DBType,
) {
    let mut operations = 0;
    let mut errors = 0;

    for entry in response.lines().filter_map(|line| {
        Blacklist::new(line, BackendType::AbuseIpDb)
    }) {
        operations += 1;
        match db_type {
            DBType::POSTGRES => {
                let conn = db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::pg::PgConnection,
                    >>()
                    .expect("Downcast failed");

                let _ = diesel::insert_into(blacklist)
                    .values(&entry)
                    .on_conflict((ip, ip_type))
                    .do_update()
                    .set(&entry)
                    .execute(conn)
                    .map_err(|err| {
                        errors += 1;
                        error!(
                            "Unable to update ip: {} with: {:#?}",
                            entry
                                .to_plain()
                                .unwrap_or_else(String::new),
                            err
                        )
                    });
            }
            DBType::MYSQL => {
                let conn = db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::mysql::MysqlConnection,
                    >>()
                    .expect("Downcast failed");

                let _ = diesel::replace_into(blacklist)
                    .values(&entry)
                    .execute(conn)
                    .map_err(|err| {
                        errors += 1;
                        error!(
                            "Unable to update ip: {} with: {:#?}",
                            entry
                                .to_plain()
                                .unwrap_or_else(String::new),
                            err
                        )
                    });
            }
            DBType::SQLITE => {
                let conn = db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::sqlite::SqliteConnection,
                    >>()
                    .expect("Downcast failed");

                let _ = diesel::replace_into(blacklist)
                    .values(&entry)
                    .execute(conn)
                    .map_err(|err| {
                        errors += 1;
                        error!(
                            "Unable to update ip: {} with: {:#?}",
                            entry
                                .to_plain()
                                .unwrap_or_else(String::new),
                            err
                        )
                    });
            }
        }
    }
    debug!(
        "Store Completed. {} Inserts or Updates, {} Errors",
        operations - errors,
        errors
    );
}

fn update_old_ips(
    args: &CliArguments,
    db_conn: &dyn Any,
    db_type: DBType,
    client: &Client,
) {
    let mut updated = 0;
    let mut deleted = 0;
    let mut update_error = 0;
    let mut delete_error = 0;

    let update_threshold = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::weeks(i64::from(
            args.expiration_days,
        )))
        .expect("Unable to substract weeks")
        .naive_utc();
    let filter = blacklist
        .filter(
            last_update
                .lt(update_threshold)
                .and(backend_type.eq(BackendType::AbuseIpDb)),
        )
        .order_by(last_update.desc());

    db_op!(db_conn, db_type, conn {
        update_old_ip(
            args,
            || filter.load::<Blacklist>(conn),
            |entry| {
                updated += 1;
                diesel::update(blacklist).set(entry).execute(conn)
            },
            |entry| {
                deleted += 1;
                diesel::delete(
                    blacklist.filter(
                        ip.eq(&entry.ip)
                            .and(ip_type.eq(entry.ip_type)),
                    ),
                )
                .execute(conn)
            },
            || update_error += 1,
            || delete_error += 1,
            client,
        );
    });
    debug!(
        "Update Completed. {} Updates, {} Deleted, {} Errors",
        updated - update_error,
        deleted - delete_error,
        update_error + delete_error,
    );
}

fn update_old_ip<Q, E1, E2, U, D>(
    args: &CliArguments,
    query: Q,
    mut update: U,
    mut delete: D,
    mut error_update: E1,
    mut error_delete: E2,
    client: &Client,
) where
    Q: FnOnce() -> Result<Vec<Blacklist>, diesel::result::Error>,
    U: FnMut(&Blacklist) -> Result<usize, diesel::result::Error>,
    D: FnMut(&Blacklist) -> Result<usize, diesel::result::Error>,
    E1: FnMut() -> (),
    E2: FnMut() -> (),
{
    let values = query().unwrap_or_else(|err| {
        error!("Unable to load ips with: {:#?}", err);
        Vec::new()
    });

    for mut value in values {
        match value.to_plain().and_then(|address| {
            debug!("Checking ip: {}", address);
            fetch_abuseipdb_ip(args, client, &address)
        }) {
            Some(true) => {
                value.last_update =
                    chrono::Utc::now().naive_utc();
                let _ = update(&value).map_err(|err| {
                    error_update();
                    error!(
                        "Unable to update ip: {} with: {:#?}",
                        value
                            .to_plain()
                            .unwrap_or_else(String::new),
                        err
                    )
                });
            }
            Some(false) => {
                let _ = delete(&value).map_err(|err| {
                    error_delete();
                    error!(
                        "Unable to delete ip: {} with: {:#?}",
                        value
                            .to_plain()
                            .unwrap_or_else(String::new),
                        err
                    )
                });
            }
            None => {
                warn!(
                    "Unable to fetch value for {}",
                    value.to_plain().unwrap_or_else(String::new)
                );
                return;
            }
        }
    }
}

fn fetch_abuseipdb_ip(
    args: &CliArguments,
    client: &Client,
    address: &str,
) -> Option<bool> {
    let response: String = abipdb_call!(
        client,
        &format!(
            "https://api.abuseipdb.com/api/v2/check?ipAddress={}",
            address
        ),
        "application/json",
        args
    )?;

    serde_json::from_str(&response)
        .map_err(|err| {
            error!(
                "Unable to parse check json request from abuseipdb: {}",
                err
            );
        })
        .ok()
        .and_then(|v: serde_json::Value| {
            v.as_object()
                .and_then(|v| v.get("data"))
                .and_then(|v| v.as_object())
                .and_then(|v| v.get("abuseConfidenceScore"))
                .and_then(|v| v.as_i64())
                .map(|v| v >= 100)
        })
}
