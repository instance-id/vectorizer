use simplelog::*;
use anyhow::Error;
use tch::Device;
use std::time::Instant;
use tokio::{sync::oneshot, task};
use std::{ sync::mpsc, thread::{self, JoinHandle} };

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType, SentenceEmbeddingsModel
};

use crate::SETTINGS;
use crate::data_types::{Documents, EmbeddedDocuments};

// --| Model Setup ------------------------------
// --|-------------------------------------------
type Message = (Documents, oneshot::Sender<EmbeddedDocuments>);

pub struct Model {
  sender: mpsc::SyncSender<Message>,
}

fn get_model_type(model_str: &str) -> SentenceEmbeddingsModelType {
  let model_type: SentenceEmbeddingsModelType; 

  match model_str {
    "L6" => model_type = SentenceEmbeddingsModelType::AllMiniLmL6V2,
    "L12" => model_type = SentenceEmbeddingsModelType::AllMiniLmL12V2,
    _ => model_type = SentenceEmbeddingsModelType::AllMiniLmL12V2
  }
  model_type
}

impl Model {
  pub fn spawn() -> (JoinHandle<anyhow::Result<()>>, Model) {
    let (sender, receiver) = mpsc::sync_channel(100);
    let handle = thread::spawn(move || Self::runner(receiver));
    (handle, Model { sender })
  }

  fn runner(receiver: mpsc::Receiver<Message>) -> anyhow::Result<(), Error> {
    debug!("Starting model runner");

    let settings = SETTINGS.read().unwrap();
    let model: SentenceEmbeddingsModel;

    if settings.get_bool("model.local").unwrap() {
      let path = settings.get_str("model.location")?;
      
      debug!("Loading local model from: {}", path);
      model = SentenceEmbeddingsBuilder::local(path)
          .with_device(Device::cuda_if_available())
          .create_model()?;

    } else {
      debug!("Loading remote model");
      let model_str = settings.get_str("model.location")?;
      
      model = SentenceEmbeddingsBuilder::remote(get_model_type(&model_str))
        .create_model()
        .expect("Could not load model");
    }
    
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
// pub async fn _embedding_async(documents: Documents) -> EmbeddedDocuments {
//   let handle = tokio::task::spawn_blocking(move ||  {
//     let embeds = get_text_embedding(&documents);
//     embeds
//   });
//
//   let res = handle.await.unwrap();
//   res
// }

fn _to_array(array: &[f32]) -> [f32; 1536] {
    array.try_into().expect("slice with incorrect length")
}

