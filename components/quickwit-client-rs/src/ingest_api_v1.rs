use std::fmt::Display;

use anyhow::Context;
use itertools::Itertools;
use serde::Serialize;

use crate::QuickwitClient;

#[derive(Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Commit {
    Auto,
    WaitFor,
    Force,
}

impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::WaitFor => write!(f, "wait_for"),
            Self::Force => write!(f, "force"),
        }
    }
}

#[derive(Clone)]
pub struct IngestAPIV1 {
    pub(crate) client: QuickwitClient,
}

impl IngestAPIV1 {
    pub async fn ingest_documents<Documents, Document>(
        &self,
        index_id: &str,
        documents: Documents,
        commit: Commit,
    ) -> anyhow::Result<()>
    where
        Documents: IntoIterator<Item = Document>,
        Document: Serialize,
    {
        let request_body = documents
            .into_iter()
            .map(|d| serde_json::to_string(&d).unwrap())
            .join("\n");
        let mut url = self
            .client
            .base_url
            .join(&format!("/api/v1/{index_id}/ingest"))?;
        url.set_query(Some(&format!("commit={commit}")));

        let res = self
            .client
            .client
            .post(url)
            .body(request_body)
            .send()
            .await?;

        res.error_for_status()
            .context("Ingest API returned an invalid HTTP status")?;

        Ok(())
    }
}
