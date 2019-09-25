use crate::{
    helper::method_not_allowed,
    middleware::diesel::{
        DBType, DieselPooledConnection, DieselReqExt,
        DieselTypeExt,
    },
    req_db_op, schema,
    types::blacklist::Blacklist,
};
use diesel::RunQueryDsl;
use iron::{
    error::IronError,
    headers::{Accept, ContentType},
    method::Method,
    mime::{Mime, SubLevel, TopLevel},
    status, IronResult, Request, Response,
};
use log::debug;

pub fn index(req: &mut Request) -> IronResult<Response> {
    #[allow(clippy::map_clone)]
    match req.url.path().get(1).map(|&t| t).unwrap_or("") {
        "blacklist" => blacklist(req),
        "health" => health(req),
        "system_health" => system_health(req),
        _ => Ok(Response::with(status::NotFound)),
    }
}

fn blacklist(req: &mut Request) -> IronResult<Response> {
    match &req.method {
        Method::Get => blacklist_get(req),
        _ => method_not_allowed(vec![Method::Get]),
    }
}

fn blacklist_get(req: &mut Request) -> IronResult<Response> {
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
                    return blacklist_get_json(req)
                }
                Mime(TopLevel::Text, SubLevel::Plain, _)
                | Mime(TopLevel::Text, SubLevel::Star, _) => {
                    return blacklist_get_text(req)
                }
                _ => {}
            }
        }
    }
    blacklist_get_default(req)
}

fn blacklist_get_default(
    req: &mut Request,
) -> IronResult<Response> {
    blacklist_get_json(req)
}

fn blacklist_get_json(req: &mut Request) -> IronResult<Response> {
    debug!("Serving blacklist_get_json request");
    let blacklist_query = schema::blacklist::dsl::blacklist;
    let values: Vec<Blacklist> =
        req_db_op!(req, blacklist_query, load);
    let json = serde_json::to_string(&values).map_err(|err| {
        IronError::new(err, status::InternalServerError)
    })?;
    let mut response = Response::with((status::Ok, json));
    response.headers.set(ContentType::json());
    Ok(response)
}

fn blacklist_get_text(req: &mut Request) -> IronResult<Response> {
    debug!("Serving blacklist_get_text request");
    let blacklist_query = schema::blacklist::dsl::blacklist;
    let values: Vec<Blacklist> =
        req_db_op!(req, blacklist_query, load);
    let plain = values
        .iter()
        .filter_map(Blacklist::to_plain)
        .collect::<Vec<String>>()
        .join("\n");
    let mut response = Response::with((status::Ok, plain));
    response.headers.set(ContentType::plaintext());
    Ok(response)
}

fn health(req: &mut Request) -> IronResult<Response> {
    match &req.method {
        Method::Get => health_get(req),
        _ => method_not_allowed(vec![Method::Get]),
    }
}

fn health_get(_req: &mut Request) -> IronResult<Response> {
    let mut response = Response::with((status::Ok, "true"));
    response.headers.set(ContentType::plaintext());
    Ok(response)
}

fn system_health(req: &mut Request) -> IronResult<Response> {
    match &req.method {
        Method::Get => system_health_get(req),
        _ => method_not_allowed(vec![Method::Get]),
    }
}

fn system_health_get(req: &mut Request) -> IronResult<Response> {
    match req.db_type() {
        DBType::MYSQL => {
            let _: DieselPooledConnection<
                diesel::mysql::MysqlConnection,
            > = req.db_conn().map_err(|err| {
                IronError::new(err, status::InternalServerError)
            })?;
        }
        DBType::POSTGRES => {
            let _: DieselPooledConnection<
                diesel::pg::PgConnection,
            > = req.db_conn().map_err(|err| {
                IronError::new(err, status::InternalServerError)
            })?;
        }
        DBType::SQLITE => {
            let _: DieselPooledConnection<
                diesel::sqlite::SqliteConnection,
            > = req.db_conn().map_err(|err| {
                IronError::new(err, status::InternalServerError)
            })?;
        }
    }

    let mut response = Response::with((status::Ok, "true"));
    response.headers.set(ContentType::plaintext());
    Ok(response)
}
