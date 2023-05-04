use simplelog::*;
use std::fs::File;
use serde_json::Value;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::data_types::{Documents, Document, MetaDataStore};
use crate::fragments::create_fragments_from_text;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize, Debug)]
struct IndexConfig {
  extensions: Vec<String>,
  directories: Vec<String>,
  ignored: Vec<String>,
}

impl serde::Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer, { serializer.serialize_str(self.to_string().as_ref()) }
}

type Index = BTreeMap<String, String>;

pub fn build_index(settings: &config::Config) -> Documents {
  let config_path = PathBuf::from(settings.get_str("indexer.project").unwrap());

  println!("Building index...");
  fn is_hidden(entry: &DirEntry) -> bool {
    entry
      .file_name()
      .to_str()
      .map(|s| s.starts_with("."))
      .unwrap_or(false)
  }

  let mut ignored: Vec<String> = Vec::new();
  let mut extensions: Vec<String> = Vec::new();
  let mut directories: Vec<String> = Vec::new();

  if let Ok(values) = settings.get::<Vec<String>>("indexer.ignored") { ignored = values; } 
  if let Ok(values) = settings.get::<Vec<String>>("indexer.extensions") { extensions = values; }
  if let Ok(values) = settings.get::<Vec<String>>("indexer.directories") { directories = values; }

  if directories.len() == 0 {
    directories.push(config_path.to_str().unwrap().to_owned());
  }

  let config = IndexConfig { ignored, extensions, directories };

  let mut index = Index::new();
  let mut documents = Documents::new();

  let mut metadata_store: MetaDataStore = MetaDataStore::new();
  let mut metadata: HashMap<String, Value>;

  if let Ok(store) = &settings.get_str("database.metadata") {
    metadata_store = MetaDataStore::from_json(store);
  }

  for dir in config.directories {
    if !Path::new(&dir).exists() {
      println!("ERROR: {:?} does not exist", dir);
      continue;
    }  

    let dir_path = WalkDir::new(dir);

    let mut entries = dir_path.into_iter();
    if let Some(Err(err)) = entries.next() {
      println!("ERROR: {}", err);
      continue;
    }

    loop {
      let entry = match entries.next() {
        None => break,
        Some(Err(err)) => { println!("ERROR: {}", err); continue; }
        Some(Ok(entry)) => entry,
      };


      // --| Skip ignored directories
      if config.ignored.len() > 0 {
        let path = entry.path().canonicalize().unwrap().to_str().unwrap().to_owned();
        let ignore = path.contains(config.ignored.to_owned().join("/").as_str());

        if ignore {  
          entries.skip_current_dir(); continue;
        }
      }

      // --| Skip hidden
      if is_hidden(&entry) {
        if entry.file_type().is_dir() { entries.skip_current_dir(); }
        continue;
      }

      // --| Check for proper file extensions 
      let extension = &entry.path().extension();
      if extension.is_none() && config.extensions.len() > 0
        && !config.extensions.contains(&"*".to_owned()) {
          continue;
      }

      if let Some(extension) = extension {
        let ext = &extension.to_str().unwrap().to_owned();
        if !config.extensions.contains(&ext) { continue; }
      }

      // --| Apply metadata to the document
      metadata = metadata_store.metadata.clone();
     
      let content = std::fs::read_to_string(entry.path()).unwrap();

      let path = entry.path().display().to_string();
      let name = entry.path().file_name().expect("Should be able to get file name").to_str().unwrap().to_owned();
      let extension = entry.path().extension().expect("Should be able to get the extension").to_str().unwrap().to_owned();
      let file_stem = entry.path().file_stem().expect("Should be able to get the file stem").to_str().unwrap().to_owned();

      metadata.insert("path".to_owned(), Value::String(path.clone()));
      metadata.insert("file_name".to_owned(), Value::String(name.clone()));
      metadata.insert("extension".to_owned(), Value::String(extension.clone()));
      metadata.insert("file_stem".to_owned(), Value::String(file_stem.clone()));

      let mut document = Document{
        name: name.clone(),
        id: uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, name.clone().as_bytes()).to_string(),
        text: content,
        metadata,
        fragments: vec![],
      };

      debug!("Indexing: {}", &path);
      let fragments = create_fragments_from_text(document.text.clone(), &settings);
      for i in 0..fragments.len() {
        document.add_fragment(fragments[i].clone(), i);
      }

      let path = entry.path().display().to_string();
      let file = entry.path().file_stem().unwrap().to_str().unwrap().to_owned();

      index.insert(path, file);
      documents.add(document);
    }
  }

  info!("Total documents: {}", documents.documents.len());
  documents
}

pub fn _search_files(buffer: String, app_data_dir: PathBuf,) -> Result<HashMap<String, String>, Error> {
  let index_file = File::open(app_data_dir.as_path()).unwrap();
  let index: Index = serde_json::from_reader(index_file).expect("Should be able to read content");
  let mut search_results = HashMap::<String, String>::new();

  for (path, filename) in index.into_iter().filter(|(_, v)| v.contains(&buffer)) {
    println!("Found: {:?} at {:?}", filename, path);
    search_results.insert(path, filename);
  }

  Ok(search_results) 
}

// pub fn open_file(path: String) -> Result<(), Error> {
//     let path = Path::new(&path);
//
//     match open::commands(path)[0].spawn() {
//         Ok(_) => {
//             println!("Opened {}", path.display());
//         }
//         Err(err) => return Err(Error::Io(err)),
//     }
//
//     Ok(())
// }
