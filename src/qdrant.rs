use uuid;
use anyhow::Result;
use std::collections::HashMap;
use chrono::{Local, DateTime};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::vectors_config::Config as vConfig;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{CreateCollection, SearchPoints, VectorParams, VectorsConfig, Vectors, SearchResponse, Filter, WithPayloadSelector, SearchParams, WithVectorsSelector, ReadConsistency };

use crate::data_types::EmbeddedDocuments;
use crate::vectorize::text_embedding_async;

// --| Qdrant DataTypes ---------------
// --|---------------------------------
#[derive(Debug, Clone, Default)]
pub struct SearchData{
  pub search_term: String,
  pub collection: Option<String>,
  pub vector: Option< Vec<f32>>,
  pub filter: Option<Filter>,
  pub limit: Option<u64>,
  pub with_payload: Option<WithPayloadSelector>,
  pub params: Option<SearchParams>,
  pub score_threshold: Option<f32>,
  pub offset: Option<u64>,
  pub vector_name: Option<String>,
  pub with_vectors: Option<WithVectorsSelector>,
  pub read_consistency: Option<ReadConsistency>,
}

// --| Qdrant Functions ---------------
// --|---------------------------------
pub async fn add_documents(client: QdrantClient, documents: EmbeddedDocuments) -> Result<()> {
  let mut collection_name = documents.collection.clone();

  if collection_name.is_empty() {
    collection_name = "test_collection".to_string();
  }

  let result = client.has_collection(&collection_name).await?;

  if !result {
    client
      .create_collection(&CreateCollection {
        collection_name: collection_name.clone().into(),
        vectors_config: Some(VectorsConfig {
          config: Some(vConfig::Params(VectorParams {
            size: 384,
            distance: Distance::Cosine.into(),
            hnsw_config: None,
            quantization_config: None,
          })),
        }),
        ..Default::default()
      })
    .await?;
  }
  
  let mut point_vec: Vec<PointStruct> = vec![];

  // let timestamp = std::time::SystemTime::now();
  // let time_string = format!("{:?}", timestamp);
  let now: DateTime<Local> = Local::now();
  println!("time_string: {}", now);


  for document in documents.documents {
    let id: String = document.id;
    let document_id = document.document_id;
    let name = document.name;
    let text = document.text;
    let metadata = document.metadata;

    let mut meta_vec: Vec<(String, String)> = vec![];
    for (key, value) in metadata {
      meta_vec.push((key, value.to_string()));
    }

    let meta = serde_json::to_string(&meta_vec).unwrap().to_string(); 

    let tmp_payload  = vec![
      ("id", id.clone().into()),
      ("document_id", document_id.into()),
      ("name", name.into()),
      ("text", text.into()),
      ("created_at", now.clone().to_string().into()),
      ("metadata", meta.into())
    ];

    let payload: Payload = tmp_payload.into_iter().collect::<HashMap<_, Value>>().into();

    let point_struct = PointStruct{
      id: Some(uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, id.as_bytes()).to_string().into()),
      payload: payload.into(),
      vectors: Some(Vectors::from(document.embeddings.clone().to_vec())), 
    };

    point_vec.push(point_struct);
  }

  client.upsert_points_blocking(collection_name, point_vec, None).await?;

  Ok(())
}

pub async fn search_documents(client: QdrantClient, search: SearchData) -> Result<SearchResponse> {
  let search = search.search_term;
  let limit = 4 * 13 as u64;

  let search_vector = text_embedding_async(search.clone()).await; 

  let search_points = SearchPoints {
    collection_name: "test_collection".into(),
    limit,
    vector: search_vector,
    with_payload: Some(WithPayloadSelector {
      selector_options: Some(SelectorOptions::Enable(true)),
    }),
    ..Default::default() 
  };

  let results = client.search_points(&search_points).await?;

  dbg!(&results);
  Ok(results)
}

pub async fn test_connection(client: QdrantClient) -> Result<()> {
  let collections_list = client.list_collections().await?;

  dbg!(collections_list);

  let collection_name = "test";
  client.delete_collection(collection_name).await?;

  client
    .create_collection(
      &CreateCollection {
        collection_name: collection_name.into(),
        vectors_config: Some(
          VectorsConfig {
            config: Some(vConfig::Params(VectorParams {
              size: 10,
              distance: Distance::Cosine.into(),
              hnsw_config: None,
              quantization_config: None,
            })),
          }),
          ..Default::default()
      }).await?;

  let collection_info = client.collection_info(collection_name).await?;
  dbg!(collection_info);

  let payload: Payload = vec![
    ("foo", "Bar".into()),
    ("bar", 12.into())
  ].into_iter().collect::<HashMap<_, Value>>().into();

  let points = vec![
    PointStruct::new(0, vec![12.; 10], payload)
  ];
 
  client.upsert_points_blocking(collection_name, points, None).await?;

  let search_result = client
    .search_points(
      &SearchPoints {
        collection_name: collection_name.into(),
        vector: vec![11.; 10],
        filter: None,
        limit: 10,
        with_vectors: None,
        with_payload: None,
        params: None,
        score_threshold: None,
        offset: None,
        ..Default::default()
    }).await?;

  dbg!(search_result);

  Ok(())
}
