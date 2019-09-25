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
pub enum BackendType {
    Unknown(i16),
    AbuseIpDb,
}

impl From<BackendType> for i16 {
    fn from(ip_type: BackendType) -> Self {
        Self::from(&ip_type)
    }
}

impl From<&BackendType> for i16 {
    fn from(ip_type: &BackendType) -> Self {
        match ip_type {
            BackendType::AbuseIpDb => 0,
            BackendType::Unknown(n) => *n,
        }
    }
}

impl<DB> ToSql<SmallInt, DB> for BackendType
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

impl<DB> FromSql<SmallInt, DB> for BackendType
where
    DB: Backend,
    i16: FromSql<SmallInt, DB>,
{
    fn from_sql(
        bytes: Option<&DB::RawValue>,
    ) -> deserialize::Result<Self> {
        match i16::from_sql(bytes)? {
            0 => Ok(BackendType::AbuseIpDb),
            n => Ok(BackendType::Unknown(n)),
        }
    }
}
