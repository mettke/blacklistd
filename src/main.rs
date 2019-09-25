#[allow(unused_imports)]
#[macro_use]
extern crate diesel;

mod args;
mod execute;
pub mod helper;
mod backend {
    pub mod abuseipdb;
}
mod routes {
    pub mod api;
    pub mod stats;
}
mod middleware {
    pub mod diesel;
    pub mod logger;
}
mod types {
    pub mod backend_type;
    pub mod blacklist;
    pub mod ip_type;
    pub mod stats;
}
mod schema;

use crate::{args::get_arguments, execute::execute};

fn main() {
    let args = get_arguments();
    if let Some(log_level) = args.log_level {
        simple_logger::init_with_level(log_level).unwrap();
    }
    execute(&args);
}
