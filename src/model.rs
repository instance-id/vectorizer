use anyhow::Error;
use crate::data_types::{Documents, EmbeddedDocuments};

trait Model {
  fn encode(&self, documents: Documents) -> Result<EmbeddedDocuments, Error>;
}
