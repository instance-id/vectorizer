use std::collections::HashMap;

use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// --| Arguments ----------------------
// --|---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
  pub dburl: Option<String>,
  pub project: Option<String>,
  pub metadata: Option<String>,
  pub fragments: Option<usize>,
  pub log_level: Option<String>,
  pub ignored: Option<Vec<String>>,
  pub extensions: Option<Vec<String>>,
  pub directories: Option<Vec<String>>,
}

impl Arguments {
  pub fn new() -> Self {
    Self {
      dburl: None,
      ignored: None,
      project: None,
      metadata: None,
      log_level: None,
      fragments: None,
      extensions: None,
      directories: None,
    }
  }
   
  pub fn from_matches(matches: &ArgMatches) -> Arguments  {
    let mut args = Self::new();
    args.dburl = matches.get_one::<String>("dburl").cloned();
    args.project = matches.get_one::<String>("project").cloned(); 
    args.log_level = matches.get_one::<String>("level").cloned();
    args.metadata = matches.get_one::<String>("metadata").cloned();

    args.fragments  = matches.get_one::<String>("fragment_max").cloned()
      .map(|s| s.parse::<usize>().unwrap());

    if let Some(values) = matches.get_many::<String>("extensions") {
      args.extensions = Some(values.map(|s| s.to_string()).collect());
    }

    if let Some(values) = matches.get_many::<String>("ignored") {
      args.ignored = Some(values.map(|s| s.to_string()).collect());
    }

    if let Some(values) = matches.get_many::<String>("directories") {
      args.directories = Some(values.map(|s| s.to_string()).collect());
    }

    args 
  }

  pub fn to_settings(&self, settings: &mut config::Config) {
    if let Some(value)  = &self.dburl       { let _ = &settings.set("database.url", value.clone()).unwrap(); }
    if let Some(value)  = &self.project     { let _ = &settings.set("indexer.project", value.clone()).unwrap(); }
    if let Some(values) = &self.ignored     { let _ = &settings.set("indexer.ignored", values.clone()).unwrap(); }
    if let Some(value)  = &self.metadata    { let _ = &settings.set("database.metadata", value.clone()).unwrap(); }
    if let Some(value)  = &self.log_level   { let _ = &settings.set("indexer.log_level", value.clone()).unwrap(); }
    if let Some(values) = &self.extensions  { let _ = &settings.set("indexer.extensions", values.clone()).unwrap(); }
    if let Some(values) = &self.directories { let _ = &settings.set("indexer.directories", values.clone()).unwrap(); }
    if let Some(value)  = &self.fragments   { let _ = &settings.set("database.max_tokens", value.clone().to_string()).unwrap(); }
  }
} 

// --| Metadata -----------------------
// --|---------------------------------
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaDataStore {
  pub metadata: HashMap<String, Value>
}

impl MetaDataStore {
  pub fn new() -> Self {
    Self { metadata: HashMap::new() }
  }

  pub fn from_json(json: &str) -> Self {
    let json_data: HashMap<String, Value> = serde_json::from_str(json)
      .expect("Error parsing JSON");

    Self { metadata: json_data }
  }
}

// --| EmbeddedDocuments --------------
// --|---------------------------------
#[derive(Debug, Clone)]
pub struct EmbeddedDocuments {
  pub collection: String,
  pub documents: Vec<EmbeddedDocument>,
  pub metadata: HashMap<String, Value>,
}

impl EmbeddedDocuments {
  pub fn new() -> Self {
    Self { 
      documents: Vec::new(),
      metadata: HashMap::new(),
      collection: String::new(),
    }
  }
}

// --| EmbeddedDocument ---------------
// --|---------------------------------
#[derive(Debug, Clone)]
pub struct EmbeddedDocument {
  pub id: String,
  pub document_id: String,
  pub name: String,
  pub text: String,
  pub embeddings: Vec<f32>, 
  pub metadata: HashMap<String, Value>,
}

// --| Documents ----------------------
// --|---------------------------------
#[derive(Debug, Clone)]
pub struct Documents {
  pub collection: String,
  pub documents: Vec<Document>,
  pub metadata: HashMap<String, Value>,
}

impl Documents {
  pub fn new() -> Self {
    Self { 
      documents: Vec::new(),
      metadata: HashMap::new(),
      collection: String::new(),
    }
  }

  pub fn add(&mut self, document: Document) {
    self.documents.push(document);
  }

  /// Converts a collection of documents to a collection of embedded documents 
  pub fn to_embedded(&self, documents: Vec<EmbeddedDocument>) -> EmbeddedDocuments {
    let mut embedded_documents = EmbeddedDocuments::new();
    embedded_documents.collection = self.collection.clone();
    embedded_documents.metadata = self.metadata.clone();
    embedded_documents.documents = documents;
    embedded_documents
  }
}

// --| Document -----------------------
// --|---------------------------------
#[derive(Debug, Clone)]
pub struct Document {
  pub id: String,
  pub name: String,
  pub text: String,
  pub metadata: HashMap<String, Value>,
  pub fragments: Vec<DocumentFragment>,
}

impl Document {
  pub fn add_fragment(&mut self, fragment_text: String, index: usize) {
    let mut fragment = DocumentFragment::new();
    fragment.document_id = self.id.clone();
    fragment.id = format!("{}_{}", self.id, index);
    fragment.name = self.name.clone();
    fragment.text = fragment_text;
    fragment.metadata = self.metadata.clone();
    self.fragments.push(fragment);
  }
}

// --| DocumentFragment ---------------
// --|---------------------------------
#[derive(Debug, Clone)]
pub struct DocumentFragment{
  pub id: String,
  pub document_id: String,
  pub name: String,
  pub text: String,
  pub metadata: HashMap<String, Value>,
}

impl DocumentFragment {
  pub fn new() -> Self {
    Self { 
      id: String::new(),
      document_id: String::new(),
      name: String::new(),
      text: String::new(),
      metadata: HashMap::new(),
    }
  }

  pub fn to_embedded(&self, embeddings: Vec<f32>) -> EmbeddedDocument {
    EmbeddedDocument {
      id: self.id.clone(),
      document_id: self.document_id.clone(),
      name: self.name.clone(),
      text: self.text.clone(),
      metadata: self.metadata.clone(),
      embeddings,
    }
  }
}
