use anyhow::Error;
use crate::data_types::EmbeddedDocuments;

// TODO: This is a reminder to implement additional databases via traits
trait Database {
  fn upsert(&self, documents: EmbeddedDocuments) -> Result<(), Error>;
}
