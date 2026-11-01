//! Admission plumbing; the grammar lives only in metadata_schema.
use crate::{err, json::Json, metadata_schema, IsaVersion, Object, Result};
impl Object {
    pub(crate) fn validate_metadata_types(&self) -> Result<()> {
        metadata_schema::validate("metadata", &Json::Object(self.metadata.clone()))
    }
    pub(crate) fn validate_metadata_versions(&self) -> Result<()> {
        let signature = |s: &Json| -> Result<()> {
            if s.get("abi")?.string()? != self.isa_version.as_str() {
                return err(
                    "§9.1",
                    "signature ABI version differs from object ISA version",
                );
            }
            if s.get("kind")?.string()? == "lproc" {
                for key in ["tokens", "args", "mem"] {
                    if s.object()?.contains_key(key) != (self.isa_version == IsaVersion::V09) {
                        return err(
                            "§9.1",
                            format!("lproc {key} field disagrees with ABI version"),
                        );
                    }
                }
            }
            Ok(())
        };
        for s in self.symbols()?.values() {
            signature(s.get("signature")?)?;
        }
        if let Some(imports) = self.section("imports") {
            for s in imports.object()?.values() {
                signature(s.get("signature")?)?;
            }
        }
        if let Some(c) = self.section("certificate") {
            for v in c.get("versions")?.object()?.values() {
                if v.string()? != self.isa_version.as_str() {
                    return err(
                        "§12.3",
                        "certificate version differs from object ISA version",
                    );
                }
            }
            for r in c.get("regions")?.object()?.values() {
                signature(r.get("signature")?)?;
            }
        }
        Ok(())
    }
}
