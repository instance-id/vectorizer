use anyhow::Error;
use crate::data_types::{Documents, EmbeddedDocuments};

// TODO: This is a reminder to implement additional models via traits
trait Model {
  fn encode(&self, documents: Documents) -> Result<EmbeddedDocuments, Error>;
}
