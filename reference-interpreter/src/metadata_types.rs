//! Admission plumbing; the grammar lives only in metadata_schema.
use crate::{json::Json, metadata_schema, Object, Result};
impl Object {
    pub(crate) fn validate_metadata_types(&self) -> Result<()> {
        metadata_schema::validate("metadata", &Json::Object(self.metadata.clone()))
    }
}
