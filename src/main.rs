mod qdrant;
mod indexer;
mod fragments;
mod vectorize;
mod data_types;
mod configuration;

use clap::{arg, Arg, Command};

use std::env;
use std::time::Instant;
use simplelog::*;
use std::fs::File;
use config::Config;
use std::str::FromStr;
use std::sync::RwLock;
use std::path::PathBuf;
use anyhow::{anyhow, Result, Error};
use qdrant_client::prelude::*;

use crate::data_types::Arguments;
use crate::qdrant::{test_connection, add_documents, SearchData, search_documents};
use crate::configuration::{get_config_path, default_project_settings};
use crate::vectorize::Model;

#[macro_use]
extern crate lazy_static;

lazy_static! {
  pub static ref SETTINGS: RwLock<Config> = RwLock::new(Config::default());
}

#[tokio::main]
async fn main() -> Result<(), Error> {
  let initial_perf = Instant::now();

  const VERSION: &str = env!("CARGO_PKG_VERSION");
  let settings_path = get_config_path("vectorizer");
  
  let matches = 
    Command::new("vectorizer")
    .about("Qdrant file indexer/uploader")
    .version(VERSION)
    .subcommand_required(true)
    .arg_required_else_help(true)
    .author("instance.id")

    .arg( // --| Project Path -------------------
      arg!(project: -p --project <Path> "The project root path"))

    .arg( // --| Included Extensions ------------
      arg!(extensions: -e --extensions <List> "The list of file extensions to include")
      .value_delimiter(',').use_value_delimiter(true))

    .arg( // --| Included Directories -----------
      arg!(directories: -d --directories <List> "The list of directories to include within the project root directory")
      .value_delimiter(',').use_value_delimiter(true))


    .arg( // --| Ignored Directories -----------
      arg!(ignored: -i --ignored <List> "The list of directories to ignore within the project root directory")
      .value_delimiter(',').use_value_delimiter(true))

    .arg( // --| Metadata json string
      arg!(metadata: -m --metadata <String> "The metadata to use for all files"))

    .arg( // --| Fragment Size ------------------
      arg!(fragment_max: -f --fragment_max <Size> "The maximum amount of tokens per fragment"))

    .arg( // --| Database Url -------------------
      arg!(dburl: -u --url <Address> "The database url to use. ex: http://localhost:6334"))

    .arg( // --| Log level ----------------------
      arg!(level: -l --level <Name> "The log level to use")
      // .default_value("info").default_missing_value("warn")
      .value_parser(["error", "warn", "info", "debug"]))

    .subcommand( // --| Index and upload --------
      Command::new("upload").long_flag("upload").about("Index and upload files")
      .arg(
        Arg::new("path").long("path").short('P').help("Path to the file to upload")
    ))
    .subcommand( // --| Index Only --------------
      Command::new("index").long_flag("index").about("Index files"))
    
    .subcommand( // --| Test Connection ---------
      Command::new("test").long_flag("test").about("Test Connection to Qdrant"))
    
    .subcommand( // --| Search -------------------
      Command::new("search").long_flag("search").about("Perform a test search on uploaded data")
      .arg(
        Arg::new("term").long("term").short('T').help("The search term to use")
      )).get_matches();


  let args = Arguments::from_matches(&matches);
  if let Some(level) = &args.log_level {
    if !env::var("RUST_LOG").is_ok() { 
      env::set_var("RUST_LOG", level);
    }  
  }

  init_logging(settings_path);
  
  let mut settings = SETTINGS.write().unwrap();
  args.to_settings(&mut settings);
  
  // --| Project Path -------
  check_project(&args.clone(), &mut settings)?;

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

  let client = QdrantClient::new(Some(config)).await?;

  match matches.subcommand() {

    // --| Index and Upload --------
    Some(("upload", _)) => {
      let upload_start = Instant::now();
      info!("Uploading files");

      let (_handle, model) = Model::spawn(); 

      let index_start = Instant::now();
      let documents = indexer::build_index(&settings);
      if documents.documents.len() == 0 { 
        warn!("No documents found");
        return Ok(()); 
      }

      info!("Indexing took {:?}", index_start.elapsed());

      let embed_start = Instant::now();
      let doc_embeds = model.encode(documents).await;
      info!("Embedding took {:?}", embed_start.elapsed());

      let add_start = Instant::now();
      add_documents(client, doc_embeds?.clone()).await?;  
      info!("Uploading took {:?}", add_start.elapsed());

      info!("Total upload time {:?}", upload_start.elapsed());
    },

    // --| Index -----------------
    Some(("index", _)) => {
     info!("Indexing files"); 
     let _documents = indexer::build_index(&settings);
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

  println!("Upsert Complete: {:?}", initial_perf.elapsed());
  Ok(())
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

  let logging_config = ConfigBuilder::new().set_location_level(default_level).set_time_to_local(true).build();
  let log_path = config_path.join("vectorizer.log");

  CombinedLogger::init(
    vec![
    TermLogger::new(default_level, logging_config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
    WriteLogger::new(default_level, logging_config, File::create(log_path).unwrap()),
  ]).unwrap();

  config_path
}
