use std::env;
use std::time::Instant;
use data_types::ModelLocation;
use simplelog::*;
use std::fs::File;
use config::Config;
use std::str::FromStr;
use std::sync::RwLock;
use std::path::PathBuf;
use anyhow::{anyhow, Result, Error};
use qdrant_client::prelude::*;

mod cli;
mod model;
mod qdrant;
mod indexer;
mod database;
mod fragments;
mod vectorize;
mod data_types;
mod configuration;
mod macros;


use crate::cli::cli;
use crate::data_types::Arguments;
use crate::qdrant::{test_connection, add_documents, SearchData, search_documents};
use crate::configuration::{get_system_config, default_project_settings};
use crate::vectorize::Model;

#[macro_use]
extern crate lazy_static;
lazy_static! {
  pub static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let initial_perf = Instant::now();
  let settings_path = get_system_config("vectorizer");

  // --| Build cli ----------
  let matches = cli().get_matches();
  
  let mut args = Arguments::from_matches(&matches);
  if let Some(level) = &args.log_level {
    if !env::var("RUST_LOG").is_ok() { env::set_var("RUST_LOG", level); }
  }

  // --| Logging ------------
  init_logging(settings_path);

  // --| Settings -----------
  let mut settings = SETTINGS.write().unwrap();
  debug!("{:?}", &settings);

  check_settings(&mut args, &mut settings);

  if args.to_settings(&mut settings).is_err(){
    // Early out if missing settings, but gives warning
    return Ok(())
  }
  
  // --| Project Path -------
  check_project(&args.clone(), &mut settings)?;
  debug!("{:?}", &settings);

  let config: QdrantClientConfig;
  if let Ok(url) = &settings.get_str("database.url") {
    config = QdrantClientConfig::from_url( url);  
  } else {
    error!("No database url provided");
    return Err(anyhow!("No database url provided"));
  }

  // --| Ensure necessary settings are present
  if !verify_settings(&settings).is_ok(){
    return Ok(());
  }

  drop(settings);
 
  let client = QdrantClient::new(Some(config)).await?;

  match matches.subcommand() {

    // --| Index and Upload --------
    Some(("upload", _)) => {
      let upload_start = Instant::now();
      info!("Uploading files");


      let index_start = Instant::now();
      let documents = indexer::build_index();
      if documents.documents.len() == 0 { 
        warn!("No documents found");
        return Ok(()); 
      }

      let (_handle, model) = Model::spawn(); 
      perf!("Indexing time: {:?}", index_start.elapsed());

      let embed_start = Instant::now();
      let doc_embeds = model.encode(documents).await;
      perf!("Embedding time: {:?}", embed_start.elapsed());
      debug!("{:?}", &doc_embeds);

      let add_start = Instant::now();
      add_documents(client, doc_embeds?.clone()).await?;  
      perf!("Upload time: {:?}", add_start.elapsed());

      perf!("Processing time: {:?}", upload_start.elapsed());
    },

    // --| Index -----------------
    Some(("index", _)) => {
     info!("Indexing files"); 
     let _documents = indexer::build_index();
    },
    
    // --| Test Connection --------
    Some(("test", _)) => {
      info!("Testing connection");
      test_connection(client).await?;
    },
    
    // --| Search ----------------
    Some(("search", args)) => {
      info!("Searching");

      let search_term: String;
      if let Some(term) = args.try_get_one::<String>("term").unwrap() {
        info!("Searching for {}", term);
        search_term = term.to_string();
      } else {
        warn!("You must provide a search term");
        return Ok(());
      }

      info!("Searching");
      let search_data = SearchData { 
        search_term: search_term.to_string(),
        ..Default::default() 
      };

      let _results = search_documents(client, search_data).await?;

    },
    _ => unreachable!(),
  }

  perfend!("Complete: {:?}", initial_perf.elapsed());
  Ok(())
}

// --| Check Settings ---------------------------
// --|-------------------------------------------
pub fn check_settings(args: &mut Arguments, settings: &mut config::Config){
  if args.location.is_none() {
    if let Ok(local) = settings.get_str("model.local") {
      args.location = Some(ModelLocation::Local);
      args.location_path = Some(local.to_string());
    }
     else {
       args.location = Some(ModelLocation::Remote);
     }
  }
}

// --| Check For Config File --------------------
// --|-------------------------------------------
pub fn check_project(args: &Arguments, settings: &mut config::Config) -> Result<(), Error> {
  let project: PathBuf;

  if let Some(path) = &args.project {
    project = PathBuf::from_str(path.as_str()).unwrap();
  } else {
    error!("No project path provided");
    return Err(anyhow!("No project path provided"));
  }

  if project.is_file(){
    settings.set("indexer.is_file", true).unwrap();
  }

  settings.set("indexer.project", project.to_str().unwrap()).unwrap();

  let mut vector_file = project.join(".vectorizer");

  if settings.get_bool("indexer.project_file").unwrap_or(false) {
    if !vector_file.exists() { vector_file = default_project_settings(project.clone()); }
  }


  if vector_file.exists() {
    let tmp_config = config::File::with_name(vector_file.to_str().unwrap()).format(config::FileFormat::Toml);
    settings.merge(tmp_config).unwrap();
  } else { 
    let mut cwd = get_current_working_dir();
    cwd.push(".vectorizer"); 

    if cwd.exists() {
      let tmp_config = config::File::with_name(cwd.to_str().unwrap()).format(config::FileFormat::Toml);
      settings.merge(tmp_config).unwrap();
    } else {
      error!("No .vectorizer file found");
      return Err(anyhow!("No .vectorizer file found"));
    }
  } 

  Ok(())
}


pub fn verify_settings(settings: &config::Config) -> Result<(), Error> {

  match settings.get_str("indexer.project") {
    Ok(_) => {}
    Err(_) => {
    error!("No project path provided");
    return Err(anyhow!("No project path provided"));
    }
  }

  match settings.get_str("database.url") {
    Ok(_) => {}
    Err(_) => {
      error!("No database url provided");
      return Err(anyhow!("No database url provided"));
    }
  }

  if !settings.get_bool("indexer.is_file").ok().is_some() {
    debug!("Is directory, check for extensions");
    match settings.get_array("indexer.extensions") {
      Ok(exts) => {
        if exts.len() == 0 {
          warn!("No extensions provided. Please provide the file extensions you wish to upload.");
          return Err(anyhow!("No extensions provided. Please provide the file extensions you wish to upload."));
        }
      }
      Err(_) => {
        warn!("No extensions provided. Please provide the file extensions you wish to upload.");
        return Err(anyhow!("No extensions provided. Please provide the file extensions you wish to upload."));
      }
    }
  } else { debug !("Is file, skip extension check"); }

  Ok(())
}

// --| Initialize logging -----------------------
// --|-------------------------------------------
pub fn init_logging(config_path: PathBuf) -> PathBuf {
  let mut default_level = LevelFilter::Warn;
  let settings = SETTINGS.write().unwrap();

  // --| If env variable is provided, it will override other log level settings --
  if let Ok(v) = env::var("RUST_LOG") {
    default_level = LevelFilter::from_str(&v).unwrap_or(LevelFilter::Warn);
  } else if let Ok(l) = settings.get_str("indexer.log_level") {
    default_level = LevelFilter::from_str(&l).unwrap();
  }

  let logging_config = ConfigBuilder::new()
    .set_location_level(default_level)
    .set_time_to_local(true).build();

  let log_path = config_path.join("vectorizer.log");

  CombinedLogger::init(
    vec![
    TermLogger::new(default_level, logging_config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
    WriteLogger::new(default_level, logging_config, File::create(log_path).unwrap()),
  ]).unwrap();

  config_path
}

// --| Helper functions -------------------------
// --|-------------------------------------------
fn get_current_working_dir() -> PathBuf  {
    let res = env::current_dir();
    match res {
        Ok(path) => path,
        Err(_) => PathBuf::from_str("FAILED").unwrap(),
    }
}

fn get_current_working_dir_str() -> String {
    let res = env::current_dir();
    match res {
        Ok(path) => path.into_os_string().into_string().unwrap(),
        Err(_) => "FAILED".to_string()
    }
}

