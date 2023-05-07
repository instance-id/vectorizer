use clap::{arg, Arg, Command};
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build application cli 
// This became much larger than expected
// TODO: Refactor from builder -> derive 
pub fn cli() -> Command {
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

    .arg( // --| Collection Name ----------------
      arg!(collection: -c --collection <Name> "The name of the collection in which to upload/create"))

    .arg( // --| Database Url -------------------
      arg!(dburl: -u --url <Address> "The database url to use. (ex: http://localhost:6334)"))

    .arg( // --| Metadata json string
      arg!(metadata: -m --metadata <JsonString> r#"The metadata to apply to all files (ex: --metadata='{ "key1": "value1", "key2": "value2" }')"#))

    .arg( // --| Local Model flag ---------------
      arg!(local: -l --local <Path> "Use a local model instead of a remote one (provide path: --local='path/to/model'"))

    .arg( // --| Remote Model flag --------------
      arg!(remote: -r --remote <Type> "Specify which all-MiniLM-*-v2 model type to use (automatic download - default: L12)")
      .value_parser(["L6","L12"]))

    .arg( // --| Max Tokens ---------------------
      arg!(token_max: -t --tokenmax <Size> "The maximum amount of tokens per fragment"))

    .arg( // --| Log level ----------------------
      arg!(level: -L --level <Name> "The log level to use")
      .value_parser(["error", "warn", "info", "debug"]))

    .subcommand( // --| Index and upload --------
     Command::new("upload").long_flag("upload").about("Index and upload files"))

    .subcommand( // --| Index Only --------------
      Command::new("index").long_flag("index").about("Index files"))

    .subcommand( // --| Test Connection ---------
      Command::new("test").long_flag("test").about("Test Connection to Qdrant"))

    .subcommand( // --| Search -------------------
      Command::new("search").long_flag("search").about("Perform a test search on uploaded data")
      .arg(Arg::new("term").long("term").short('T').help("The search term to use")))
}
