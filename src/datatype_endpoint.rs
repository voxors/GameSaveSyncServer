use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow, deserialize, serialize};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(
    Serialize, Deserialize, ToSchema, Debug, Clone, Copy, PartialEq, Eq, AsExpression, FromSqlRow,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
}

impl<DB> ToSql<Text, DB> for OS
where
    DB: Backend,
    str: ToSql<Text, DB>,
{
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, DB>) -> serialize::Result {
        match self {
            OS::Windows => <str as ToSql<Text, DB>>::to_sql("windows", out),
            OS::Linux => <str as ToSql<Text, DB>>::to_sql("linux", out),
        }
    }
}

impl<DB> FromSql<Text, DB> for OS
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: <DB as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        let s = <String as FromSql<Text, DB>>::from_sql(bytes)?;
        match s.as_str() {
            "windows" => Ok(OS::Windows),
            "linux" => Ok(OS::Linux),
            other => Err(format!("invalid OS value in the database: {other}").into()),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SavePathCreate {
    pub path: String,
    pub operating_system: OS,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct SavePath {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub path: SavePathCreate,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ExecutableCreate {
    pub executable: String,
    pub operating_system: OS,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Executable {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub executable: ExecutableCreate,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GameMetadataCreate {
    pub known_name: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub steam_appid: Option<String>,
    pub default_name: String,
}
#[derive(Serialize, Deserialize, ToSchema, IntoParams)]
pub struct GameMetadata {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub metadata: GameMetadataCreate,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SaveReference {
    pub uuid: String,
    pub path_id: i32,
    pub time: i64,
}
