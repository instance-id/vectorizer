use tch::Device;
use anyhow::Error;
use simplelog::*;
use std::time::Instant;
use std::{ sync::mpsc, thread::{self, JoinHandle} };

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType
};

use tokio::{sync::oneshot, task};
use crate::data_types::{Documents, EmbeddedDocuments};

type Message = (Documents, oneshot::Sender<EmbeddedDocuments>);

pub struct Model {
  sender: mpsc::SyncSender<Message>,
}

impl Model {
  pub fn spawn() -> (JoinHandle<anyhow::Result<()>>, Model) {
    let (sender, receiver) = mpsc::sync_channel(100);
    let handle = thread::spawn(move || Self::runner(receiver));
    (handle, Model { sender })
  }

  fn runner(receiver: mpsc::Receiver<Message>) -> anyhow::Result<(), Error> {
    let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL12V2)
      .create_model()
      .expect("Could not load model");

    // let model = SentenceEmbeddingsBuilder::local("resources/all-MiniLM-L6-v2")
    //     .with_device(Device::cuda_if_available())
    //     .create_model()?;

    while let Ok((documents, sender)) = receiver.recv() {
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

      sender.send(documents.to_embedded(embedded_documents)).expect("sending results");
    }

    Ok(())
  }

  pub async fn encode(&self, documents: Documents) -> Result<EmbeddedDocuments, Error> {
    let (sender, receiver) = oneshot::channel();
    task::block_in_place(|| self.sender.send((documents, sender)))?;
    Ok(receiver.await?)
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

fn _to_array(array: &[f32]) -> [f32; 1536] {
    array.try_into().expect("slice with incorrect length")
}

