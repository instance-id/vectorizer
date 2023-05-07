use anyhow::Error;
use crate::data_types::EmbeddedDocuments;

trait Database {
  fn upsert(&self, documents: EmbeddedDocuments) -> Result<(), Error>;
}
