use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, TS, Clone, Copy, Debug, Default, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OrderDirection {
    Asc,
    #[default]
    Desc,
}
