use std::sync::Arc;

use anyhow::Context;
use itertools::Itertools;
use opentelemetry::json::OTELLogDocument;
use quickwit_client_rs::{
    search_api_v1::{self, SearchRequest},
    QuickwitClient,
};
use rand::seq::SliceRandom;
use uuid::Uuid;

use crate::domain::ports::logs::{
    LogsSearcher, SearchLogsOpts, SearchLogsOutput, SearchLogsQuery, SearchLogsQueryClause,
};

#[derive(Clone)]
pub struct LogsSearcherAdapter {
    pub searcher_clients: Arc<Vec<search_api_v1::SearchAPIV1>>,
}

impl LogsSearcherAdapter {
    pub fn new(urls: Vec<String>) -> anyhow::Result<Self> {
        let mut searcher_clients = Vec::new();
        for url in urls {
            let client = QuickwitClient::new(url)?;
            searcher_clients.push(client.search_api_v1());
        }
        Ok(Self {
            searcher_clients: Arc::new(searcher_clients),
        })
    }
}

impl LogsSearcher for LogsSearcherAdapter {
    #[tracing::instrument(skip(self), err)]
    async fn search_logs<'a>(
        &self,
        organization_id: Uuid,
        options: SearchLogsOpts<'a>,
    ) -> anyhow::Result<SearchLogsOutput> {
        // pick a searcher. It could be any searcher from the list since Quickwit handles load balancing internally anyway.
        let dest_client = self
            .searcher_clients
            .choose(&mut rand::thread_rng())
            .context("failed to pick a destination searcher")?;

        // compute the name of the desintation index from the id of the organization
        // we isolate logs from multiple tenants using the organisation id, and add a prefix with a version number to leave open the possibility of future developments
        let destination_index_name = format!("{}-logs-index-v1", organization_id);

        let quickwit_request = SearchRequest {
            query: serialize_query(&options.query),

            // Quickwit's API required that start_timestamp and end_timestamp be in seconds, even if it can ingest logs with nanoseconds precision
            start_timestamp: options.from_timestamp.map(|ts| ts.timestamp()),
            end_timestamp: options.to_timestamp.map(|ts| ts.timestamp()),
            start_offset: options.start_offset,
            max_hits: options.max_hits,
            search_fields: None,
            snippet_fields: None,
            sort_by: Some("-timestamp_nanos".to_string()),
        };

        tracing::debug!(request = ?quickwit_request, "executing quickwik request");
        println!("{:#?}", quickwit_request);

        let response = dest_client
            .search(&destination_index_name, &quickwit_request)
            .await
            .context("Failed to search logs")?;

        // convert the quickwit response to OTELLogDocument
        let logs = response
            .hits
            .into_iter()
            .map(|document| {
                let document_value = serde_json::Value::Object(document);
                serde_json::from_value::<OTELLogDocument>(document_value)
                    .context("Failed to desrialize raw quickwit document to OTEL Log Document")
            })
            .collect::<anyhow::Result<Vec<_>>>()?;

        //

        let output = SearchLogsOutput {
            total_hits: response.num_hits,
            logs,
        };

        Ok(output)
    }
}

fn serialize_query<'a>(query: &'a SearchLogsQuery<'a>) -> String {
    match query {
        SearchLogsQuery::Or(a, b) => format!("({} OR {})", serialize_query(a), serialize_query(b)),
        SearchLogsQuery::And(a, b) => {
            format!("({} AND {})", serialize_query(a), serialize_query(b))
        }
        SearchLogsQuery::Not(query) => format!("-({})", serialize_query(query)),
        SearchLogsQuery::Clause(clause) => serialize_clause(clause),
    }
}

fn serialize_clause<'a>(clause: &'a SearchLogsQueryClause<'a>) -> String {
    match clause {
        SearchLogsQueryClause::Term { field_name, term } => format!("{field_name}:{term}"),
        SearchLogsQueryClause::TermPrefix { field_name, term } => format!("{field_name}:{term}*"),
        SearchLogsQueryClause::TermSet { field_name, terms } => {
            format!("{}:IN [{}]", field_name, terms.iter().join(" "))
        }
        SearchLogsQueryClause::Phrase {
            field_name,
            phrase,
            slop,
        } => format!("{field_name}:\"{phrase}\"~{}", slop.unwrap_or_default()),
        SearchLogsQueryClause::PhrasePrefix {
            field_name,
            phrase_prefix,
        } => format!("{field_name}:\"{phrase_prefix}\"*"),
        SearchLogsQueryClause::FieldExists { field_name } => format!("{field_name}:*"),
        SearchLogsQueryClause::MatchAll => "*".to_string(),
    }
}
