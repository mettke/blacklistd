use crate::{
    helper::method_not_allowed,
    middleware::diesel::{
        DBType, DieselPooledConnection, DieselReqExt,
        DieselTypeExt,
    },
    req_db_op,
    schema::blacklist::dsl::*,
    types::stats::IpsPerTime,
};
use diesel::{
    query_builder::SqlQuery, sql_query, QueryDsl, RunQueryDsl,
};
use iron::{
    error::IronError,
    headers::{Accept, ContentType},
    method::Method,
    mime::{Mime, SubLevel, TopLevel},
    status, IronResult, Request, Response,
};
use log::debug;
use serde_json::json;

pub fn index(req: &mut Request) -> IronResult<Response> {
    #[allow(clippy::map_clone)]
    match req.url.path().get(1).map(|&t| t).unwrap_or("") {
        "count" => count(req),
        "countPerDay" => count_per_day(req),
        _ => Ok(Response::with(status::NotFound)),
    }
}

fn count(req: &mut Request) -> IronResult<Response> {
    match &req.method {
        Method::Get => count_get(req),
        _ => method_not_allowed(vec![Method::Get]),
    }
}

fn count_get(req: &mut Request) -> IronResult<Response> {
    if let Some(Accept(mimes)) = req.headers.get() {
        let mut mimes = mimes.clone();
        mimes.sort_by(|a, b| b.quality.cmp(&a.quality));
        for mime in mimes {
            match mime.item {
                Mime(
                    TopLevel::Application,
                    SubLevel::Json,
                    _,
                )
                | Mime(
                    TopLevel::Application,
                    SubLevel::Star,
                    _,
                )
                | Mime(TopLevel::Star, _, _) => {
                    return count_get_json(req)
                }
                Mime(TopLevel::Text, SubLevel::Plain, _)
                | Mime(TopLevel::Text, SubLevel::Star, _) => {
                    return count_get_text(req)
                }
                _ => {}
            }
        }
    }
    count_get_default(req)
}

fn count_get_default(req: &mut Request) -> IronResult<Response> {
    count_get_json(req)
}

fn count_get_json(req: &mut Request) -> IronResult<Response> {
    debug!("Serving count_get_json request");
    let count_query = blacklist.count();
    let count: i64 = req_db_op!(req, count_query, get_result);
    let count = json!({ "count": count });
    let json = serde_json::to_string(&count).map_err(|err| {
        IronError::new(err, status::InternalServerError)
    })?;
    let mut response = Response::with((status::Ok, json));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn count_get_text(req: &mut Request) -> IronResult<Response> {
    debug!("Serving count_get_text request");
    let count_query = blacklist.count();
    let count: i64 = req_db_op!(req, count_query, get_result);
    let mut response =
        Response::with((status::Ok, count.to_string()));
    response.headers.set(ContentType::plaintext());
    Ok(response)
}

fn count_per_day(req: &mut Request) -> IronResult<Response> {
    match &req.method {
        Method::Get => count_per_day_get(req),
        _ => method_not_allowed(vec![Method::Get]),
    }
}

fn count_per_day_get(req: &mut Request) -> IronResult<Response> {
    if let Some(Accept(mimes)) = req.headers.get() {
        let mut mimes = mimes.clone();
        mimes.sort_by(|a, b| b.quality.cmp(&a.quality));
        for mime in mimes {
            match mime.item {
                Mime(
                    TopLevel::Application,
                    SubLevel::Json,
                    _,
                )
                | Mime(
                    TopLevel::Application,
                    SubLevel::Star,
                    _,
                )
                | Mime(TopLevel::Star, _, _) => {
                    return count_per_day_get_json(req)
                }
                Mime(TopLevel::Text, SubLevel::Plain, _)
                | Mime(TopLevel::Text, SubLevel::Star, _) => {
                    return count_per_day_get_text(req)
                }
                _ => {}
            }
        }
    }
    count_per_day_get_default(req)
}

fn count_per_day_query(req: &mut Request) -> SqlQuery {
    match req.db_type() {
        DBType::MYSQL => {
            sql_query("
                SELECT
                    COUNT(*) as count, 
                    MIN(last_update) as last_update_start, 
                    MAX(last_update) as last_update_end
                FROM blacklist
                GROUP BY UNIX_TIMESTAMP(last_update) DIV 86400
                ORDER BY last_update_start DESC;
            ")
        }
        DBType::POSTGRES => {
            sql_query("
                SELECT
                    COUNT(*) as count, 
                    MIN(last_update) as last_update_start, 
                    MAX(last_update) as last_update_end
                FROM blacklist
                GROUP BY round(extract('epoch' from last_update) / 86400)
                ORDER BY last_update_start DESC;
            ")
        },
        DBType::SQLITE => {
            sql_query("
                SELECT
                    COUNT(*) as count, 
                    MIN(last_update) as last_update_start, 
                    MAX(last_update) as last_update_end
                FROM blacklist
                GROUP BY datetime((strftime('%s', last_update) / 86400) * 86400, 'unixepoch')
                ORDER BY last_update_start DESC;
            ")
        },
    }
}

fn count_per_day_get_default(
    req: &mut Request,
) -> IronResult<Response> {
    count_per_day_get_json(req)
}

fn count_per_day_get_json(
    req: &mut Request,
) -> IronResult<Response> {
    debug!("Serving count_per_day_get_json request");
    let count_query = count_per_day_query(req);
    let count: Vec<IpsPerTime> =
        req_db_op!(req, count_query, get_results);
    let json = serde_json::to_string(&count).map_err(|err| {
        IronError::new(err, status::InternalServerError)
    })?;
    let mut response = Response::with((status::Ok, json));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn count_per_day_get_text(
    req: &mut Request,
) -> IronResult<Response> {
    debug!("Serving count_per_day_get_text request");
    let count_query = count_per_day_query(req);
    let count: Vec<IpsPerTime> =
        req_db_op!(req, count_query, get_results);
    let plain: String = count
        .into_iter()
        .map(|s| {
            format!(
                "{} {} {}",
                &s.count.to_string(),
                &s.last_update_start.to_string(),
                &s.last_update_end.to_string()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let mut response = Response::with((status::Ok, plain));
    response.headers.set(ContentType::plaintext());
    Ok(response)
}
