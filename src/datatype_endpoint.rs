use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{AsExpression, FromSqlRow, deserialize, serialize};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(ToSchema)]
#[allow(unused)]
pub struct UploadedSave {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    #[schema(value_type = String, example = json!([{"relative_path": "file.txt", "hash": "abc123"}]))]
    pub file_hash: Vec<FileHash>,
}

#[derive(ToSchema)]
#[allow(unused)]
pub struct UploadedFileYaml {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(
    Serialize,
    Deserialize,
    ToSchema,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    AsExpression,
    FromSqlRow,
    Hash,
)]
#[diesel(sql_type = Text)]
#[serde(rename_all = "lowercase")]
pub enum OS {
    Windows,
    Linux,
    Undefined,
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
            OS::Undefined => <str as ToSql<Text, DB>>::to_sql("undefined", out),
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
            "undefined" => Ok(OS::Undefined),
            other => Err(format!("invalid OS value in the database: {other}").into()),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct SavePathCreate {
    pub path: String,
    pub operating_system: OS,
}
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct SavePath {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub path: SavePathCreate,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ExecutableCreate {
    pub executable: String,
    pub operating_system: OS,
}
#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct Executable {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub executable: ExecutableCreate,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct GameMetadataCreate {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub known_name: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub steam_appid: Option<String>,
    pub default_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub install_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub gog: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub flatpak_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub lutris_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub epic_cloud: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub gog_cloud: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub origin_cloud: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub steam_cloud: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(required = false, nullable)]
    pub uplay_cloud: Option<bool>,
    pub gog_extra: Option<Vec<i64>>,
    pub steam_extra: Option<Vec<i64>>,
}

#[derive(Serialize, Deserialize, ToSchema, IntoParams, Clone)]
pub struct GameMetadata {
    pub id: Option<i32>,
    #[serde(flatten)]
    pub metadata: GameMetadataCreate,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct FileHash {
    pub relative_path: String,
    pub hash: String,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct SaveReference {
    pub uuid: String,
    pub path_id: i32,
    pub time: i64,
    pub files_hash: Vec<FileHash>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct GameMetadataWithPaths {
    pub game_metadata: GameMetadata,
    pub paths: Vec<SavePath>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct ConfigurationForm {
    pub value: String,
}
