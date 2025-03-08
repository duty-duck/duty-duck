use chrono::{DateTime, Utc};
use opentelemetry::{json::OTELLogDocument, proto::logs::v1::ResourceLogs};
use uuid::Uuid;

use crate::domain::entities::authorization::AuthContext;

/// A port for the ingestor service in charge of ingesting logs (in the Open Telemetry format)
#[async_trait::async_trait]
pub trait LogsIngestor {
    /// Ingests logs into the system. Accepts [ResourceLogs] from the opentelemetry-proto crate
    /// because the ingestor's main API uses gRPC
    async fn ingest_logs(
        &self,
        auth_context: &AuthContext,
        logs: Vec<ResourceLogs>,
    ) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct SearchLogsOpts<'a> {
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
    pub start_offset: u64,
    pub max_hits: u64,
    pub query: SearchLogsQuery<'a>,
}

/// The representation of simple query language to search logs, modeled after Quickwit's query language
/// (Although, since we use ports and adapters to abstract Quickwit, we could implement this query language on top of another search engine someday)
#[derive(Clone, Debug)]
#[allow(unused)]
pub enum SearchLogsQuery<'a> {
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Not(Box<Self>),
    Clause(SearchLogsQueryClause<'a>),
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum SearchLogsQueryClause<'a> {
    Term {
        field_name: &'a str,
        term: &'a str,
    },
    TermPrefix {
        field_name: &'a str,
        term: &'a str,
    },
    TermSet {
        field_name: &'a str,
        terms: &'a [&'a str],
    },
    Phrase {
        field_name: &'a str,
        phrase: &'a str,
        slop: Option<usize>,
    },
    PhrasePrefix {
        field_name: &'a str,
        phrase_prefix: &'a str,
    },
    FieldExists {
        field_name: &'a str,
    },
    MatchAll,
}

pub struct SearchLogsOutput {
    pub logs: Vec<OTELLogDocument>,
    pub total_hits: u64,
}

/// A port for the service in charge of retreiveing logs (in the Open Telemetry format)
pub trait LogsSearcher {
    /// Searches logs matchign the provided options
    async fn search_logs<'a>(
        &self,
        organization_id: Uuid,
        options: SearchLogsOpts<'a>,
    ) -> anyhow::Result<SearchLogsOutput>;
}
