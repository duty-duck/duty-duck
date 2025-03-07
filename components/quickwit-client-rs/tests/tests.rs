use chrono::Utc;
use quickwit_client_rs::{
    indexes_api_v1::{
        otel::{otel_logs_doc_mapping, OTELLogInput, OTELResource},
        DateTimeFieldMapping, DocMapping, DocMappingMode, FieldMapping, IndexConfig,
        RetentionSettings, SearchSettings, TextFieldMapping,
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
    let resource = {
        let mut attributes = serde_json::Map::new();
        attributes.insert(
            "service.name".to_string(),
            serde_json::Value::String("crawler".to_string()),
        );
        Some(OTELResource {
            attributes,
            dropped_attributes_count: 0,
        })
    };
    let documents = vec![
        OTELLogInput {
            timestamp_unix: Some(now.timestamp()),
            observed_timestamp_unix: now.timestamp(),
            resource: resource.clone(),

            severity_text: Some("debug".to_string()),
            severity_number: Some(5),
            body: Some(
                serde_json::to_value(OTELLogBody {
                    message: "hello there",
                })
                .unwrap(),
            ),

            ..Default::default()
        },
        OTELLogInput {
            timestamp_unix: Some(now.timestamp()),
            observed_timestamp_unix: now.timestamp(),
            resource: resource.clone(),
            severity_text: Some("debug".to_string()),
            severity_number: Some(5),
            body: Some(
                serde_json::to_value(OTELLogBody {
                    message: "general kenobi",
                })
                .unwrap(),
            ),
            ..Default::default()
        },
        OTELLogInput {
            timestamp_unix: Some(now.timestamp()),
            observed_timestamp_unix: now.timestamp(),
            resource,
            severity_text: Some("error".to_string()),
            severity_number: Some(18),
            body: Some(
                serde_json::to_value(OTELLogBody {
                    message: "SOMETHING WENT WRONG",
                })
                .unwrap(),
            ),
            ..Default::default()
        },
    ];

    // ingest documents
    client
        .ingest_api_v1()
        .ingest_documents(&index_id, documents, Commit::Force)
        .await?;

    // search documents by message
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

    // search documents by severity
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

    // search documents by service
    let res = client
        .search_api_v1()
        .search(
            &index_id,
            &SearchRequest {
                query: "resource_attributes.service.name:crawler".to_string(),
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

    assert_eq!(res.num_hits, 3);
    assert_eq!(res.hits.len(), 3);
    println!("{:#?}", res.hits);

    Ok(())
}
