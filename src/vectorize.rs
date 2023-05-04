use simplelog::*;
use std::time::Instant;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType, SentenceEmbeddingsModel,
};

use crate::data_types::{Documents, EmbeddedDocuments};

pub struct Model {
  pub model: SentenceEmbeddingsModel,
}

impl Model {
  pub async fn new() -> tokio::task::JoinHandle<Self> {
    let handle = tokio::task::spawn_blocking(move ||  {
      let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
        .create_model()
        .expect("Could not create model");
      Self { model }
    });

    handle  
  }
}

// --| Text Embedding -----------------
// --|---------------------------------
pub async fn text_embedding_async(text: String) -> Vec<f32> {
  let handle = tokio::task::spawn_blocking(move ||  {
    let embeds = get_text_embedding(&text);
    embeds
  });

  let res = handle.await.unwrap();
  res
}

pub fn get_text_embedding(text: &str) -> Vec<f32> {
  let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
    .create_model()
    .expect("Could not create model");
  
  let embedding = model.encode(&[text.to_string()]).expect("Could not embed fragment");
  embedding[0].clone()
}

// --| Documents ----------------------
// --|---------------------------------
pub async fn _embedding_async(documents: Documents) -> EmbeddedDocuments {
  let handle = tokio::task::spawn_blocking(move ||  {
    let embeds = _get_embeddings3(&documents);
    embeds
  });

  let res = handle.await.unwrap();
  res
}

pub async fn get_embeddings(documents: &Documents, handle: tokio::task::JoinHandle<Model>) -> EmbeddedDocuments  {

  let model_start = Instant::now();
  let model = handle.await.unwrap().model;
  info!("Model created in {:?}", model_start.elapsed());
  
  let mut embedded_documents = Vec::new();

  let mut document_average_time = vec![];

  let documents_start = Instant::now();
  for document in &documents.documents {
    for fragment in &document.fragments {
      let doc_start = Instant::now();
      
      let embedding = model.encode(&[fragment.text.clone()]).expect("Could not embed fragment");
      embedded_documents.push(fragment.to_embedded(embedding[0].clone()));

      document_average_time.push(doc_start.elapsed());
    }
  }
  info!("Documents embedded in {:?}", documents_start.elapsed());

  let mut total_time = 0;
  for time in &document_average_time {
    total_time += time.as_millis();
  }

  let total_items = &document_average_time.len();

  let average_time = total_time / *total_items as u128;
  info!("Average time per document: {}ms", average_time);
  info!("Total Items: {}", total_items);

  documents.to_embedded(embedded_documents)
}

/// Embeds a collection of documents using the AllMiniLmL12V2 model
pub fn _get_embeddings3(documents: &Documents) -> EmbeddedDocuments  {
  let model_start = Instant::now();

  let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
    .create_model()
    .expect("Could not create model");

  info!("Model created in {:?}", model_start.elapsed());
  
  let mut embedded_documents = Vec::new();

  let mut document_average_time = vec![];

  let documents_start = Instant::now();
  for document in &documents.documents {
    for fragment in &document.fragments {
      let doc_start = Instant::now();
      
      let embedding = model.encode(&[fragment.text.clone()]).expect("Could not embed fragment");
      embedded_documents.push(fragment.to_embedded(embedding[0].clone()));

      document_average_time.push(doc_start.elapsed());
    }
  }
  info!("Documents embedded in {:?}", documents_start.elapsed());

  let mut total_time = 0;
  for time in &document_average_time {
    total_time += time.as_millis();
  }

  let total_items = &document_average_time.len();

  let average_time = total_time / *total_items as u128;
  info!("Average time per document: {}ms", average_time);
  info!("Total Items: {}", total_items);

  documents.to_embedded(embedded_documents)
}

fn _to_array(array: &[f32]) -> [f32; 1536] {
    array.try_into().expect("slice with incorrect length")
}
