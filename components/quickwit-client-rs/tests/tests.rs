use chrono::Utc;
use quickwit_client_rs::{
    indexes_api_v1::{
        otel_logs_doc_mapping, DateTimeFieldMapping, DocMapping, DocMappingMode, FieldMapping,
        IndexConfig, RetentionSettings, SearchSettings, TextFieldMapping,
    },
    ingest_api_v1::Commit,
    search_api_v1::SearchRequest,
    QuickwitClient,
};
use serde::Serialize;

#[derive(Serialize)]
struct TestDocument {
    content: &'static str,
    ts: chrono::DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OTELLogBody {
    message: &'static str,
}

#[derive(Serialize)]
struct OTELLogEvent {
    timestamp_nanos: i64,
    observed_timestamp_nanos: i64,
    service_name: &'static str,
    severity_text: &'static str,
    severity_number: u64,
    body: OTELLogBody,
}

#[tokio::test]
async fn integration_test_1() -> anyhow::Result<()> {
    let index_id = "integration_test_index".to_string();
    let client = QuickwitClient::new("http://quickwit:7280")?;

    // delete index from previous runs
    let _ = client.indexes_api_v1().delete_index(&index_id).await;

    // create index
    client
        .indexes_api_v1()
        .create_index(&IndexConfig {
            index_id: index_id.clone(),
            version: quickwit_client_rs::indexes_api_v1::Version::V08,
            retention: Some(RetentionSettings {
                period: "1 day".to_string(),
                schedule: "hourly".to_string(),
            }),
            doc_mapping: DocMapping {
                mode: DocMappingMode::Dynamic,
                timestamp_field: Some("ts".to_string()),
                field_mappings: vec![
                    FieldMapping::Text(TextFieldMapping {
                        name: "content".to_string(),
                        description: None,
                        stored: true,
                        indexed: true,
                        tokenizer: None,
                        fieldnorms: false,
                        record: None,
                        fast: false,
                    }),
                    FieldMapping::DateTime(DateTimeFieldMapping {
                        name: "ts".to_string(),
                        description: None,
                        stored: true,
                        indexed: true,
                        fast: true,
                        fast_precision: None,
                        input_formats: None,
                        output_format: None,
                    }),
                ],
                tag_fields: None,
                store_source: false,
                partition_key: None,
                max_num_partitions: None,
            },
            search_settings: None,
            indexing_settings: None,
        })
        .await?;

    // write documents
    let documents = vec![
        TestDocument {
            content: "De cœur il photosensibles Dans maison quelqu'un personne poids permettait.",
            ts: Utc::now(),
        },
        TestDocument {
            content: "Portes en vers vous blocaus vous les vous suis moment vers sont effet toutes coupé blocaus coupé portes toutes repris.",
            ts: Utc::now(),
        },
    ];

    // ingest documents
    client
        .ingest_api_v1()
        .ingest_documents(&index_id, documents, Commit::Force)
        .await?;

    // search document
    let res = client
        .search_api_v1()
        .search(
            &index_id,
            &SearchRequest {
                query: "content:blocaus".to_string(),
                start_timestamp: None,
                end_timestamp: None,
                start_offset: 0,
                max_hits: 10,
                search_fields: None,
                snippet_fields: None,
                sort_by: None,
            },
        )
        .await?;

    assert_eq!(res.num_hits, 1);
    assert_eq!(res.hits.len(), 1);
    println!("{:#?}", res.hits);

    Ok(())
}

#[tokio::test]
async fn integration_test_otel_logs_1() -> anyhow::Result<()> {
    let index_id = "integration_test_otel_logs_index".to_string();
    let client = QuickwitClient::new("http://quickwit:7280")?;

    // delete index from previous runs
    let _ = client.indexes_api_v1().delete_index(&index_id).await;

    // create index
    client
        .indexes_api_v1()
        .create_index(&IndexConfig {
            index_id: index_id.clone(),
            version: quickwit_client_rs::indexes_api_v1::Version::V08,
            retention: None,
            doc_mapping: otel_logs_doc_mapping(),
            search_settings: Some(SearchSettings {
                default_search_fields: Some(vec!["body.message".to_string()]),
            }),
            indexing_settings: None,
        })
        .await?;

    // write documents
    let now = Utc::now();
    let documents = vec![
        OTELLogEvent {
            timestamp_nanos: now.timestamp_millis(),
            observed_timestamp_nanos: now.timestamp_millis(),
            service_name: "crawler",
            severity_text: "debug",
            severity_number: 0,
            body: OTELLogBody {
                message: "hello there",
            },
        },
        OTELLogEvent {
            timestamp_nanos: now.timestamp_millis(),
            observed_timestamp_nanos: now.timestamp_millis(),
            service_name: "crawler",
            severity_text: "debug",
            severity_number: 0,
            body: OTELLogBody {
                message: "general kenobi",
            },
        },
        OTELLogEvent {
            timestamp_nanos: now.timestamp_millis(),
            observed_timestamp_nanos: now.timestamp_millis(),
            service_name: "crawler",
            severity_text: "error",
            severity_number: 5,
            body: OTELLogBody {
                message: "SOMETHING WENT WRONG",
            },
        },
    ];

    // ingest documents
    client
        .ingest_api_v1()
        .ingest_documents(&index_id, documents, Commit::Force)
        .await?;

    // search documents
    let res = client
        .search_api_v1()
        .search(
            &index_id,
            &SearchRequest {
                query: "\"something went wrong\"".to_string(),
                start_timestamp: None,
                end_timestamp: None,
                start_offset: 0,
                max_hits: 10,
                search_fields: None,
                snippet_fields: None,
                sort_by: None,
            },
        )
        .await?;

    assert_eq!(res.num_hits, 1);
    assert_eq!(res.hits.len(), 1);
    println!("{:#?}", res.hits);

    let res = client
        .search_api_v1()
        .search(
            &index_id,
            &SearchRequest {
                query: "severity_text:debug".to_string(),
                start_timestamp: None,
                end_timestamp: None,
                start_offset: 0,
                max_hits: 10,
                search_fields: None,
                snippet_fields: None,
                sort_by: None,
            },
        )
        .await?;

    assert_eq!(res.num_hits, 2);
    assert_eq!(res.hits.len(), 2);
    println!("{:#?}", res.hits);

    Ok(())
}
