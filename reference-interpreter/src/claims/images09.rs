//! K85 fixed classical images are metadata bindings, never observations of an
//! unknown quantum region. Decode the exact packed cell image and hash bytes.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FixedRegionBinding {
    /// §12.2 K85: signature mem slot, independent of local region spelling.
    pub id: String,
    pub group: usize,
    pub slot: usize,
    pub address_width: u64,
    pub cell_width: u64,
    pub sha256: String,
}

pub(super) struct ImageDomain {
    pub binding: FixedRegionBinding,
    name: String,
    values: Vec<BitVector>,
}

impl ImageDomain {
    pub fn unrestricted_entry(&self) -> Result<Json> {
        let mut cells = crate::reserved(self.values.len(), "§12.2 K85", "fixed region cells")?;
        for (index, value) in self.values.iter().enumerate() {
            let (lo, hi) = (BitVector::zero(value.width())?, ones(value.width())?);
            cells.push(Json::Object(BTreeMap::from([
                ("index".into(), Json::Number(index.to_string())),
                ("lo".into(), decimal_json(&lo)),
                ("hi".into(), decimal_json(&hi)),
            ])));
        }
        Ok(Json::Object(BTreeMap::from([
            ("name".into(), Json::String(self.name.clone())),
            ("cells".into(), Json::Array(cells)),
        ])))
    }

    pub fn append_cells(&self, cells: &mut Vec<(Component, BitVector)>) -> Result<()> {
        crate::reserve_more(cells, self.values.len(), "§12.2 K85", "fixed cells")?;
        cells.extend(self.values.iter().enumerate().map(|(index, value)| {
            (
                Component::Cell(self.binding.group, self.binding.slot, index),
                value.clone(),
            )
        }));
        Ok(())
    }
}

impl Object {
    pub(crate) fn validate_image_domains09(&self, domain: &Json, f: &Function) -> Result<()> {
        self.image_domains09(domain, f).map(|_| ())
    }

    pub(super) fn image_domains09(&self, domain: &Json, f: &Function) -> Result<Vec<ImageDomain>> {
        let Some(images) = domain.object()?.get("fixed_regions") else {
            return Ok(Vec::new());
        };
        let mut resolved = BTreeMap::new();
        for image in images.array()? {
            let name = image.get("name")?.string()?;
            let (group, slot) = interface_slot(f, name)?;
            let param = &f.params[group][slot];
            let Type::Region(a, w) = param.ty else {
                return err("§12.2 K85", "fixed image is not an interface region");
            };
            if param.role.as_deref().unwrap_or("in") != "in"
                || image.get("A")?.uint()? != a as u128
                || image.get("W")?.uint()? != w as u128
            {
                return err(
                    "§12.2 K85",
                    "fixed image requires matching in-region dimensions",
                );
            }
            if domain
                .get("regions")?
                .array()?
                .iter()
                .any(|r| r.get("name").and_then(Json::string) == Ok(name))
                || resolved.contains_key(&(group, slot))
            {
                return err(
                    "§12.2 K85",
                    "fixed region occurs more than once in the domain",
                );
            }
            let count = crate::oracle::power_of_two(a as u128)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| {
                    crate::resource_error("§12.2 K85", "fixed region cell count exceeds usize")
                })?;
            let width = usize::try_from(w)
                .map_err(|_| crate::resource_error("§12.2 K85", "cell width exceeds usize"))?;
            let bits = count.checked_mul(width).ok_or_else(|| {
                crate::resource_error("§12.2 K85", "fixed image bit count exceeds usize")
            })?;
            let encoded = image.get("image")?.string()?;
            if encoded.len()
                != bits.div_ceil(8).checked_mul(2).ok_or_else(|| {
                    crate::resource_error("§12.2 K85", "fixed image text exceeds usize")
                })?
                || !encoded
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return err(
                    "§12.2 K85",
                    "fixed image requires exact lowercase hexadecimal bytes",
                );
            }
            let mut bytes = crate::reserved(bits.div_ceil(8), "§12.2 K85", "fixed image bytes")?;
            for pair in encoded.as_bytes().as_chunks::<2>().0 {
                let digit = |b: u8| {
                    if b.is_ascii_digit() {
                        b - b'0'
                    } else {
                        b - b'a' + 10
                    }
                };
                bytes.push((digit(pair[0]) << 4) | digit(pair[1]));
            }
            if bits % 8 != 0 && bytes.last().is_some_and(|b| *b >> (bits % 8) != 0) {
                return err("§12.2 K85", "fixed image has nonzero unused high bits");
            }
            let hash = crate::sha256_hex(&bytes);
            if image.get("sha256")?.string()? != hash {
                return err("§12.2 K85", "fixed image SHA256 differs from raw bytes");
            }
            let mut values = crate::reserved(count, "§12.2 K85", "fixed image cells")?;
            for cell in 0..count {
                let mut value = BitVector::zero(width)?;
                for bit in 0..width {
                    let offset = cell * width + bit;
                    value.set(bit, bytes[offset / 8] >> (offset % 8) & 1 != 0)?;
                }
                values.push(value);
            }
            resolved.insert(
                (group, slot),
                ImageDomain {
                    binding: FixedRegionBinding {
                        id: format!("mem:{slot}"),
                        group,
                        slot,
                        address_width: a,
                        cell_width: w,
                        sha256: hash,
                    },
                    name: name.into(),
                    values,
                },
            );
        }
        Ok(resolved.into_values().collect())
    }
}
