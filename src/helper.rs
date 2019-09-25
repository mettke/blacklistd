use crate::types::ip_type::IpType;
use iron::{
    headers::Allow, method::Method, status, IronResult, Response,
};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn convert_ip(
    raw_ip: &[u8],
    ip_type: IpType,
) -> Option<IpAddr> {
    match ip_type {
        IpType::Ipv4 if raw_ip.len() >= 4 => {
            let mut input = [0; 4];
            input.copy_from_slice(&raw_ip[0..4]);
            Some(IpAddr::V4(Ipv4Addr::from(input)))
        }
        IpType::Ipv6 if raw_ip.len() >= 16 => {
            let mut input = [0; 16];
            input.copy_from_slice(&raw_ip[0..16]);
            Some(IpAddr::V6(Ipv6Addr::from(input)))
        }
        _ => None,
    }
}

pub fn method_not_allowed(
    allowed_methods: Vec<Method>,
) -> IronResult<Response> {
    let mut response = Response::with(status::MethodNotAllowed);
    response.headers.set(Allow(allowed_methods));
    Ok(response)
}

pub fn get_elapsed_time(time: time::Tm) -> i64 {
    let response_time = time::now() - time;
    response_time.num_seconds()
}

#[macro_export]
macro_rules! req_db_op {
    ($req:expr, $query:expr, $func:ident) => {
        match $req.db_type() {
            DBType::MYSQL => {
                let conn: DieselPooledConnection<
                    diesel::mysql::MysqlConnection,
                > = $req.db_conn().map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?;
                $query.$func(&conn).map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?
            }
            DBType::POSTGRES => {
                let conn: DieselPooledConnection<
                    diesel::pg::PgConnection,
                > = $req.db_conn().map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?;
                $query.$func(&conn).map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?
            }
            DBType::SQLITE => {
                let conn: DieselPooledConnection<
                    diesel::sqlite::SqliteConnection,
                > = $req.db_conn().map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?;
                $query.$func(&conn).map_err(|err| {
                    IronError::new(
                        err,
                        status::InternalServerError,
                    )
                })?
            }
        }
    };
}

#[macro_export]
macro_rules! db_op {
    ($db_conn:expr, $db_type:expr, $conn:ident $op: block) => {
        match $db_type {
            DBType::MYSQL => {
                let $conn = $db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::mysql::MysqlConnection,
                    >>()
                    .expect("Downcast failed");

                $op
            }
            DBType::POSTGRES => {
                let $conn = $db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::pg::PgConnection,
                    >>()
                    .expect("Downcast failed");

                $op
            }
            DBType::SQLITE => {
                let $conn = $db_conn
                    .downcast_ref::<DieselPooledConnection<
                        diesel::sqlite::SqliteConnection,
                    >>()
                    .expect("Downcast failed");

                $op
            }
        }
    };
}
