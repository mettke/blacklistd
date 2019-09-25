use crate::{
    helper::convert_ip,
    schema::blacklist,
    types::{backend_type::BackendType, ip_type::IpType},
};
use chrono::Utc;
use serde::{Serialize, Serializer};
use std::net::IpAddr;

#[derive(
    Debug,
    Clone,
    Hash,
    Queryable,
    Insertable,
    Identifiable,
    AsChangeset,
)]
#[table_name = "blacklist"]
#[primary_key(ip, ip_type)]
pub struct Blacklist {
    pub ip: Vec<u8>,
    pub ip_type: IpType,
    pub backend_type: BackendType,
    pub last_update: chrono::NaiveDateTime,
}

impl Blacklist {
    pub fn new(
        ip: &str,
        backend_type: BackendType,
    ) -> Option<Self> {
        let (ip, ip_type) = match ip.parse() {
            Ok(IpAddr::V4(addr)) => {
                let octets = addr.octets().to_vec();
                (octets, IpType::Ipv4)
            }
            Ok(IpAddr::V6(addr)) => {
                let octets = addr.octets().to_vec();
                (octets, IpType::Ipv6)
            }
            Err(_) => return None,
        };
        let last_update = Utc::now().naive_utc();
        Some(Blacklist {
            ip,
            ip_type,
            backend_type,
            last_update,
        })
    }
}

impl Serialize for Blacklist {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match convert_ip(&self.ip, self.ip_type) {
            Some(ref value) => serializer.serialize_some(value),
            None => serializer.serialize_none(),
        }
    }
}

impl Blacklist {
    pub fn to_plain(&self) -> Option<String> {
        convert_ip(&self.ip, self.ip_type)
            .as_ref()
            .map(IpAddr::to_string)
    }
}
