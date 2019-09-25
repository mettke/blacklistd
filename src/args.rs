use crate::middleware::diesel::DBType;
use clap::{
    app_from_crate, crate_authors, crate_description, crate_name,
    crate_version, Arg, ArgMatches,
};
use log::{error, Level};
use std::process::exit;

const ARGS_LISTEN: &str = "listen";
const ARGS_LISTEN_DEFAULT: &str = "::";
const ARGS_PORT: &str = "port";
const ARGS_PORT_DEFAULT: &str = "8080";
const ARGS_PORT_DEFAULT_U16: u16 = 8080;
const ARGS_VERBOSE: &str = "verbose";
const ARGS_SILENT: &str = "silent";

const ARGS_EXPIRATION_DAYS: &str = "expiration-days";
const ARGS_EXPIRATION_DAYS_DEFAULT: &str = "14";
const ARGS_EXPIRATION_DAYS_DEFAULT_U32: u32 = 14;
const ARGS_STALE_DAYS: &str = "stale-days";
const ARGS_STALE_DAYS_DEFAULT: &str = "28";
const ARGS_STALE_DAYS_DEFAULT_U32: u32 = 28;

const ARGS_DATABASE_TYPE: &str = "db-type";
const ARGS_DATABASE_HOST: &str = "db-host";
const ARGS_DATABASE_PORT: &str = "db-port";
const ARGS_DATABASE_NAME: &str = "db-name";
const ARGS_DATABASE_USER: &str = "db-user";
const ARGS_DATABASE_PASS: &str = "db-pass";
const ARGS_DATABASE_PATH: &str = "db-path";

const ARGS_BACKEND_ABUSEIPDB: &str = "api-abuseipdb";

#[derive(Debug, Clone)]
pub struct CliArguments {
    pub listen: String,
    pub port: u16,
    pub log_level: Option<Level>,

    pub expiration_days: u32,
    pub stale_days: u32,

    pub db_type: DBType,
    pub db_host: String,
    pub db_port: u16,
    pub db_name: String,
    pub db_user: String,
    pub db_pass: String,
    pub db_path: String,

    pub api_abuseipdb: Option<String>,
}

pub fn get_arguments() -> CliArguments {
    let matches = get_cli_config();
    let listen = matches
        .value_of(ARGS_LISTEN)
        .unwrap_or(ARGS_LISTEN_DEFAULT)
        .into();
    let port: u16 = matches
        .value_of(ARGS_PORT)
        .and_then(|port| port.parse().ok())
        .unwrap_or(ARGS_PORT_DEFAULT_U16);
    let log_level = match (
        matches.occurrences_of(ARGS_SILENT),
        matches.occurrences_of(ARGS_VERBOSE),
    ) {
        (0, 1) => Some(Level::Debug),
        (0, v) if v > 1 => Some(Level::Trace),
        (1, 0) => Some(Level::Warn),
        (2, 0) => Some(Level::Error),
        (s, 0) if s > 2 => None,
        _ => Some(Level::Info),
    };

    let expiration_days: u32 = matches
        .value_of(ARGS_EXPIRATION_DAYS)
        .and_then(|e| e.parse().ok())
        .unwrap_or(ARGS_EXPIRATION_DAYS_DEFAULT_U32);
    let stale_days: u32 = matches
        .value_of(ARGS_STALE_DAYS)
        .and_then(|e| e.parse().ok())
        .unwrap_or(ARGS_STALE_DAYS_DEFAULT_U32);

    let db_type = match matches
        .value_of(ARGS_DATABASE_TYPE)
        .and_then(|t| DBType::parse(t))
    {
        Some(v) => v,
        None => {
            error!("Database Type is required");
            exit(1);
        }
    };
    let db_host = match matches.value_of(ARGS_DATABASE_HOST) {
        Some(v) => v.into(),
        None => {
            error!("Database Host is required");
            exit(1);
        }
    };
    let db_port = match matches
        .value_of(ARGS_DATABASE_PORT)
        .and_then(|port| port.parse().ok())
    {
        Some(v) => v,
        None => {
            error!("Database Port is required");
            exit(1);
        }
    };
    let db_name = match matches.value_of(ARGS_DATABASE_NAME) {
        Some(v) => v.into(),
        None => {
            error!("Database Name is required");
            exit(1);
        }
    };
    let db_user = match matches.value_of(ARGS_DATABASE_USER) {
        Some(v) => v.into(),
        None => {
            error!("Database User is required");
            exit(1);
        }
    };
    let db_pass = match matches.value_of(ARGS_DATABASE_PASS) {
        Some(v) => v.into(),
        None => {
            error!("Database Password is required");
            exit(1);
        }
    };
    let db_path = match matches.value_of(ARGS_DATABASE_PATH) {
        Some(v) => v.into(),
        None => {
            error!("Database Path is required");
            exit(1);
        }
    };

    let api_abuseipdb = matches
        .value_of(ARGS_BACKEND_ABUSEIPDB)
        .map(|v| v.into());

    CliArguments {
        listen,
        port,
        log_level,
        expiration_days,
        stale_days,
        db_type,
        db_host,
        db_port,
        db_name,
        db_user,
        db_pass,
        db_path,
        api_abuseipdb,
    }
}

fn get_cli_config<'a>() -> ArgMatches<'a> {
    app_from_crate!()
        .arg(
            Arg::with_name(ARGS_LISTEN)
                .short("l")
                .long(ARGS_LISTEN)
                .value_name("hostname/ip")
                .help("Set the listening ip/hostname")
                .default_value(ARGS_LISTEN_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_PORT)
                .short("p")
                .long(ARGS_PORT)
                .value_name("port")
                .help("Set the port to bind to")
                .default_value(ARGS_PORT_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_VERBOSE)
                .short("v")
                .long(ARGS_VERBOSE)
                .conflicts_with(ARGS_SILENT)
                .multiple(true)
                .help("Increase verbosity. Once for debug, twice for trace")
        )
        .arg(
            Arg::with_name(ARGS_SILENT)
                .short("s")
                .long(ARGS_SILENT)
                .conflicts_with(ARGS_VERBOSE)
                .multiple(true)
                .help("Decrease verbosity. Once for warning, twice for error, thrice for none")
        )
        .arg(
            Arg::with_name(ARGS_EXPIRATION_DAYS)
                .long(ARGS_EXPIRATION_DAYS)
                .value_name("days")
                .help("Days until ip recheck")
                .default_value(ARGS_EXPIRATION_DAYS_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_STALE_DAYS)
                .long(ARGS_STALE_DAYS)
                .value_name("days")
                .help("Days until ip removale")
                .default_value(ARGS_STALE_DAYS_DEFAULT)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_TYPE)
                .long(ARGS_DATABASE_TYPE)
                .value_name("type")
                .possible_values(&["postgres", "mysql", "sqlite"])
                .help("Database Type")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_HOST)
                .long(ARGS_DATABASE_HOST)
                .value_name("hostname/ip")
                .help("Database Hostname or Ip")
                .takes_value(true)
                .required_ifs(&[
                    (ARGS_DATABASE_TYPE, "postgres"),
                    (ARGS_DATABASE_TYPE, "mysql"),
                ])
                .default_value_if(
                    ARGS_DATABASE_TYPE,
                    Some("sqlite"),
                    "",
                ),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_PORT)
                .long(ARGS_DATABASE_PORT)
                .value_name("port")
                .help("Database Port")
                .takes_value(true)
                .required_ifs(&[
                    (ARGS_DATABASE_TYPE, "postgres"),
                    (ARGS_DATABASE_TYPE, "mysql"),
                ])
                .default_value_if(
                    ARGS_DATABASE_TYPE,
                    Some("sqlite"),
                    "0",
                ),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_NAME)
                .long(ARGS_DATABASE_NAME)
                .value_name("name")
                .help("Database Name")
                .takes_value(true)
                .required_ifs(&[
                    (ARGS_DATABASE_TYPE, "postgres"),
                    (ARGS_DATABASE_TYPE, "mysql"),
                ])
                .default_value_if(
                    ARGS_DATABASE_TYPE,
                    Some("sqlite"),
                    "",
                ),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_USER)
                .long(ARGS_DATABASE_USER)
                .value_name("username")
                .help("Database Username")
                .takes_value(true)
                .required_ifs(&[
                    (ARGS_DATABASE_TYPE, "postgres"),
                    (ARGS_DATABASE_TYPE, "mysql"),
                ])
                .default_value_if(
                    ARGS_DATABASE_TYPE,
                    Some("sqlite"),
                    "",
                ),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_PASS)
                .long(ARGS_DATABASE_PASS)
                .value_name("password")
                .help("Database Password")
                .takes_value(true)
                .required_ifs(&[
                    (ARGS_DATABASE_TYPE, "postgres"),
                    (ARGS_DATABASE_TYPE, "mysql"),
                ])
                .default_value_if(
                    ARGS_DATABASE_TYPE,
                    Some("sqlite"),
                    "",
                ),
        )
        .arg(
            Arg::with_name(ARGS_DATABASE_PATH)
                .long(ARGS_DATABASE_PATH)
                .value_name("path")
                .help("Database Path")
                .takes_value(true)
                .required_ifs(&[(ARGS_DATABASE_TYPE, "sqlite")])
                .default_value_ifs(&[
                    (ARGS_DATABASE_TYPE, Some("postgres"), ""),
                    (ARGS_DATABASE_TYPE, Some("mysql"), ""),
                ]),
        )
        .arg(
            Arg::with_name(ARGS_BACKEND_ABUSEIPDB)
                .long(ARGS_BACKEND_ABUSEIPDB)
                .value_name("abuseipdb-api")
                .help("API Key for abuseipdb")
                .takes_value(true)
                // For future addition of new backends
                .required_unless_one(&[]),
        )
        .get_matches()
}
