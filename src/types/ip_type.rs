use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    serialize::{self, Output, ToSql},
    sql_types::SmallInt,
};
use std::io::Write;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    AsExpression,
    FromSqlRow,
    Hash,
    Eq,
)]
#[sql_type = "SmallInt"]
pub enum IpType {
    Unknown(i16),
    Ipv4,
    Ipv6,
}

impl From<IpType> for i16 {
    fn from(ip_type: IpType) -> Self {
        Self::from(&ip_type)
    }
}

impl From<&IpType> for i16 {
    fn from(ip_type: &IpType) -> Self {
        match ip_type {
            IpType::Ipv4 => 0,
            IpType::Ipv6 => 1,
            IpType::Unknown(n) => *n,
        }
    }
}

impl<DB> ToSql<SmallInt, DB> for IpType
where
    DB: Backend,
    i16: ToSql<SmallInt, DB>,
{
    fn to_sql<W: Write>(
        &self,
        out: &mut Output<W, DB>,
    ) -> serialize::Result {
        let n: i16 = self.into();
        n.to_sql(out)
    }
}

impl<DB> FromSql<SmallInt, DB> for IpType
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(
        bytes: Option<&DB::RawValue>,
    ) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(IpType::Ipv4),
            1 => Ok(IpType::Ipv6),
            n => Ok(IpType::Unknown(n)),
        }
    }
}
