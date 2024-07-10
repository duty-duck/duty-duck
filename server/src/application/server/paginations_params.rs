use serde::Deserialize;
use ts_rs::TS;

#[derive(Deserialize, Debug, TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PaginationParams {
    page_number: Option<u32>,
    items_per_page: Option<u32>,
}

impl PaginationParams {
    pub fn page_number(&self) -> u32 {
        self.page_number.unwrap_or(1)
    }

    #[inline]
    pub fn items_per_page(&self) -> u32 {
        self.items_per_page.unwrap_or(20)
    }
}