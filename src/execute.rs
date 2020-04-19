use crate::{
    args::CliArguments,
    db_op,
    middleware::{
        diesel::{
            DBType, DieselMiddleware, DieselPool,
            DieselPooledConnection,
        },
        logger::Logger,
    },
    routes::{api, stats},
    schema::blacklist::dsl::*,
    types::blacklist::Blacklist,
};
use clokwerk::{ScheduleHandle, Scheduler, TimeUnits};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use iron::{status, Chain, Iron, IronResult, Request, Response};
use log::{debug, error, info, trace};
use reqwest::blocking::Client;
use std::{any::Any, process::exit, time::Duration};

fn request(req: &mut Request) -> IronResult<Response> {
    trace!("Handling request: {:?}", req);
    #[allow(clippy::map_clone)]
    match req.url.path().get(0).map(|&t| t).unwrap_or("") {
        "api" => api::index(req),
        "stats" => stats::index(req),
        _ => Ok(Response::with(status::NotFound)),
    }
}

fn setup_diesel_and_schedular(
    args: &CliArguments,
    chain: &mut Chain,
) -> ScheduleHandle {
    let url = format!(
        "{}://{}:{}@{}:{}/{}",
        args.db_type,
        args.db_user,
        args.db_pass,
        args.db_host,
        args.db_port,
        args.db_name
    );
    match args.db_type {
        DBType::POSTGRES => {
            let conn = create_database_connection::<
                diesel::pg::PgConnection,
            >(&url, args.db_type);
            let pool = conn.pool.clone();
            chain.link_before(conn);
            setup_scheduler(args, pool, args.db_type)
        }
        DBType::MYSQL => {
            let conn = create_database_connection::<
                diesel::mysql::MysqlConnection,
            >(&url, args.db_type);
            let pool = conn.pool.clone();
            chain.link_before(conn);
            setup_scheduler(args, pool, args.db_type)
        }
        DBType::SQLITE => {
            let conn = create_database_connection::<
                diesel::sqlite::SqliteConnection,
            >(&args.db_path, args.db_type);
            let pool = conn.pool.clone();
            chain.link_before(conn);
            setup_scheduler(args, pool, args.db_type)
        }
    }
}

fn create_database_connection<T: diesel::Connection>(
    url: &str,
    db_type: DBType,
) -> DieselMiddleware<T> {
    match DieselMiddleware::new(url, db_type) {
        Err(err) => {
            error!("Unable to connect to database: {}", err);
            exit(4);
        }
        Ok(dm) => dm,
    }
}

fn setup_scheduler<T: 'static + diesel::Connection>(
    args: &CliArguments,
    db_pool: DieselPool<T>,
    db_type: DBType,
) -> ScheduleHandle {
    if let Some(db_conn) = db_pool.try_get() {
        let db_conn: &dyn Any = &db_conn;
        let update_necessary: bool = db_op!(db_conn, db_type, conn {
            let stale_threshold = chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::days(1))
                .expect("Unable to substract 1 day")
                .naive_utc();
            let values = blacklist
                .order_by(last_update.desc())
                .limit(1)
                .load::<Blacklist>(conn)
                .ok().and_then(|mut v| v.pop());
            match values {
                Some(ref value) if value.last_update > stale_threshold => false,
                _ => true
            }
        });
        if update_necessary {
            update_blacklists(&args, db_pool.clone(), db_type);
        }
    } else {
        error!("Connection to database lost");
    }

    let args = args.clone();
    let mut scheduler = Scheduler::new();
    scheduler.every(1.day()).at("12:00 am").run(move || {
        update_blacklists(&args, db_pool.clone(), db_type)
    });
    scheduler.watch_thread(Duration::from_secs(60))
}

fn update_blacklists<T: 'static + diesel::Connection>(
    args: &CliArguments,
    db_pool: DieselPool<T>,
    db_type: DBType,
) {
    info!("Updating blacklist");

    let client = Client::new();
    let db_conn: DieselPooledConnection<T> =
        match db_pool.try_get() {
            Some(conn) => conn,
            None => {
                error!("Connection to database lost");
                return;
            }
        };

    if args.api_abuseipdb.is_some() {
        crate::backend::abuseipdb::update_abuseipdb(
            args, &db_conn, db_type, &client,
        );
    }
    debug!("Deleting stale ips");
    delete_old_ips(args, &db_conn, db_type);
}

fn delete_old_ips(
    args: &CliArguments,
    db_conn: &dyn Any,
    db_type: DBType,
) {
    let removale_threshold = chrono::Utc::now()
        .checked_sub_signed(chrono::Duration::days(i64::from(
            args.stale_days,
        )))
        .expect("Unable to substract days")
        .naive_utc();

    let deleted = db_op!(db_conn, db_type, conn {
        diesel::delete(
                blacklist.filter(
                    last_update
                        .lt(removale_threshold),
                ),
            ).execute(conn) .map_err(|err| {
                error!("Unable to delete ips with: {:#?}", err)
            })
            .unwrap_or(0)
    });
    debug!("Removale Completed. {} Deleted", deleted);
}

pub fn execute(args: &CliArguments) {
    let mut chain = Chain::new(request);
    let (logger_before, logger_after) = Logger::new();
    chain.link_before(logger_before);
    let scheduler = setup_diesel_and_schedular(&args, &mut chain);

    chain.link_after(logger_after);

    info!("Starting server on {}:{}", args.listen, args.port);
    if let Err(err) = Iron::new(chain)
        .http(format!("{}:{}", args.listen, args.port))
    {
        error!("Unable to start Server: {}", err);
    }
    scheduler.stop();
}
