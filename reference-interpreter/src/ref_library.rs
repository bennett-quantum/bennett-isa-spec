//! Checked §11.1 ref@0.9 binding. Constructor semantics are shared with the
//! independent, std-only conformance crate; no Python or subprocess is used.
//! This layer does not infer source bodies or certify a supplied registry's
//! agreement with linked source (§9.2). Source lowering remains a separate job.
use crate::{err, IsaVersion, Object, Result, ScheduleExpansion};
use bennett_isa_conformance::{
    json,
    ref_hash::{canonical, envelope_hash},
    ref_language::Expander,
};
pub use bennett_isa_conformance::{
    json::Json as Value,
    ref_language::{Leaf, Registry},
};

pub const REF09_HASH: &str = "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197";
const CATALOG: &str = include_str!("../../spec/ref-0.9.json");

/// Only the published complete library can satisfy a ref@0.9 binding. Private
/// witness catalogs cannot acquire its identity by supplying its descriptor.
#[derive(Clone, Debug)]
pub struct RefLibrary {
    expander: Expander,
}
fn infeasible(e: impl std::fmt::Display) -> crate::Error {
    let text = e.to_string();
    if let Some(reason) = text.strip_prefix(bennett_isa_conformance::ref_language::HOST_RESOURCE_LIMIT_PREFIX) {
        crate::Error::host_resource(format!("§11.1: {reason}"))
    } else {
        crate::Error::new(format!("§11.1: E_SCHEDULE_INFEASIBLE: {text}"))
    }
}
impl RefLibrary {
    pub fn published() -> Result<Self> {
        static LIBRARY: std::sync::OnceLock<Result<RefLibrary>> = std::sync::OnceLock::new();
        LIBRARY
            .get_or_init(|| Self::bind("ref", "0.9", REF09_HASH, CATALOG))
            .clone()
    }
    pub fn bind(name: &str, version: &str, hash: &str, bytes: &str) -> Result<Self> {
        if (name, version, hash) != ("ref", "0.9", REF09_HASH) {
            return Err(infeasible("bound expansion library is unavailable"));
        }
        if envelope_hash(bytes) != hash {
            return Err(infeasible("catalog envelope does not match binding"));
        }
        let value = json::parse(bytes).map_err(infeasible)?;
        if canonical(&value).map_err(infeasible)? != bytes {
            return Err(infeasible(
                "catalog canonical bytes/envelope do not match binding",
            ));
        }
        let expander = Expander::new(bytes, Registry::new()).map_err(infeasible)?;
        Ok(Self { expander })
    }
    pub fn hash(&self) -> &'static str {
        REF09_HASH
    }

    /// Expand typed actuals to leaves at a constructor fixpoint. Registry
    /// entries must already be source-bound by the caller, including K77's
    /// per-invocation construction. An expansion is local and errors publish
    /// no leaves or partially mutated expander state.
    pub fn expand(
        &self,
        kind: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        registry: Registry,
    ) -> Result<Vec<Leaf>> {
        self.expander
            .with_registry(registry)
            .and_then(|mut e| e.expand(kind, args, bundles))
            .map_err(|e| crate::Error::new(format!("§11.1: {e}")))
    }
}
impl Object {
    /// Expand a checked structured source invocation with captured classical
    /// actuals. Source adjoints are formed before constructor instantiation.
    pub fn expand_action_source(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
    ) -> Result<Vec<Leaf>> {
        self.ensure_linked()?;
        self.check_hashes()?;
        let f = self
            .functions
            .get(name)
            .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
            .ok_or_else(|| crate::Error::new("§11.1: source action is not defined".into()))?;
        let library = self.bound_ref_library()?;
        let identity = self.identity()?;
        let hash = identity.closure_hashes[&f.name]
            .clone()
            .ok_or_else(|| crate::Error::new("§11.1: source action closure is unlinked".into()))?;
        let registry = self
            .functions
            .values()
            .filter(|f| matches!(f.kind.as_str(), "oracle" | "action" | "perm_action"))
            .map(|f| {
                let h = identity.closure_hashes[&f.name]
                    .clone()
                    .ok_or_else(|| crate::Error::new("§11.1: source closure is unlinked".into()))?;
                Ok((
                    h,
                    Value::Obj(std::collections::BTreeMap::from([
                        ("parameters".into(), Value::Arr(source_parameters(self, f)?)),
                        ("body".into(), Value::Arr(Vec::new())),
                    ])),
                ))
            })
            .collect::<Result<Registry>>()?;
        let factory =
            std::sync::Arc::new(crate::source_constructor::LinkedSource::new(self.clone())?);
        let mut e = library
            .expander
            .with_registry(registry)
            .map_err(infeasible)?
            .with_source_factory(factory);
        e.expand_bound(
            "action_call",
            &[Value::Str("$root".into()), Value::Arr(args.to_vec())],
            bundles,
            &[bennett_isa_conformance::ref_language::SourceSymbol {
                name: "$root".into(),
                hash,
                oracle_invocation: None,
                adjoint,
            }],
        )
        .map_err(infeasible)
    }
    /// Resolve a concrete ref library without modifying canonical metadata.
    /// No 0.9 library is substituted for an accepted 0.8 binding.
    pub fn bound_ref_library(&self) -> Result<RefLibrary> {
        if self.isa_version != IsaVersion::V09 {
            return err(
                "§11.1",
                "E_SCHEDULE_INFEASIBLE: ref library for this ISA version is unavailable",
            );
        }
        let binding = self.schedule_expansion()?.unwrap_or(ScheduleExpansion {
            name: "ref",
            version: "0.9",
            hash: Some(REF09_HASH),
        });
        if (binding.name, binding.version, binding.hash) != ("ref", "0.9", Some(REF09_HASH)) {
            return Err(infeasible("bound expansion library is unavailable"));
        }
        RefLibrary::published()
    }
}

/// A source-bound K77 channel, captured with its positional actual vector.
/// Cloning/forwarding this value keeps its tuple; expansion has no ambient
/// current-token state for a nested caller to overwrite. It intentionally
/// stores no process-local token id and does not add a signature argument.
#[derive(Clone, Debug)]
pub struct OracleInvocation<'a> {
    object: &'a Object,
    function: &'a crate::Function,
    closure_hash: String,
    budget: crate::Budget,
    context: Value,
    arguments: Vec<Value>,
}
impl<'a> OracleInvocation<'a> {
    /// `source_budget` is resolved from the source tk (or explicit root
    /// request), independently of the supplied catalog context. Callers retain
    /// the source invocation identity alongside this capture, not in the hash.
    pub fn capture(
        object: &'a Object,
        name: &str,
        context: Option<&Value>,
        source_budget: crate::Budget,
        arguments: &[Value],
    ) -> Result<Self> {
        let invalid = |e| crate::Error::new(format!("§11.1 K77: {e}"));
        if object.isa_version != IsaVersion::V09 {
            return Err(invalid("oracle context requires an ISA 0.9 object"));
        }
        let function = object
            .functions
            .get(name)
            .filter(|f| f.kind == "oracle")
            .ok_or_else(|| invalid("source binding must name a linked oracle"))?;
        let context = context.ok_or_else(|| invalid("missing oracle_invocation"))?;
        let Value::Obj(fields) = context else {
            return Err(invalid("ill-typed oracle_invocation"));
        };
        if fields.len() != 1 {
            return Err(invalid("oracle_invocation must contain only token"));
        }
        let tuple = fields
            .get("token")
            .and_then(Value::as_arr)
            .filter(|v| v.len() == 4)
            .ok_or_else(|| invalid("token requires four decimal strings"))?;
        let expected = [
            source_budget.steps.to_string(),
            source_budget.heap.to_string(),
            source_budget.stack.to_string(),
            source_budget.depth.to_string(),
        ];
        for (part, expected) in tuple.iter().zip(&expected) {
            let part = part
                .as_str()
                .ok_or_else(|| invalid("token component must be a string"))?;
            if part.is_empty()
                || !part.bytes().all(|b| b.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
            {
                return Err(invalid("noncanonical token component"));
            }
            if part != expected {
                return Err(invalid("source-inconsistent oracle_invocation"));
            }
        }
        object
            .validate_budget(source_budget)
            .map_err(|e| e.context("§11.1 K77"))?;
        if arguments.len() != function.params.iter().map(Vec::len).sum::<usize>() {
            return Err(invalid(
                "argument vector must contain exactly the declared parameters",
            ));
        }
        object.check_hashes()?;
        let closure_hash = object.identity()?.closure_hashes[name]
            .clone()
            .ok_or_else(|| invalid("unresolved linked closure"))?;
        Ok(Self {
            object,
            function,
            closure_hash,
            budget: source_budget,
            context: context.clone(),
            arguments: arguments.to_vec(),
        })
    }
    /// Construct and expand this checked linked source invocation. No supplied
    /// callback or basis-input witness participates in source construction.
    pub fn expand_source(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
    ) -> Result<Vec<Leaf>> {
        self.expand_source_with_leaf_limit(library, bundles, bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT)
    }
    /// Use a caller-selected materialisation capacity. This changes host
    /// capacity only; the source token and its ISA fault boundaries are intact.
    pub fn expand_source_with_leaf_limit(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
        leaf_limit: usize,
    ) -> Result<Vec<Leaf>> {
        let registry = self
            .object
            .functions
            .values()
            .filter(|f| matches!(f.kind.as_str(), "oracle" | "action" | "perm_action"))
            .map(|f| {
                let hash = self.object.identity()?.closure_hashes[&f.name]
                    .clone()
                    .ok_or_else(|| crate::Error::new("§11.1: unresolved source closure".into()))?;
                Ok((
                    hash,
                    Value::Obj(std::collections::BTreeMap::from([
                        (
                            "parameters".into(),
                            Value::Arr(source_parameters(self.object, f)?),
                        ),
                        ("body".into(), Value::Arr(Vec::new())),
                    ])),
                ))
            })
            .collect::<Result<Registry>>()?;
        let factory = std::sync::Arc::new(crate::source_constructor::LinkedSource::new(
            self.object.clone(),
        )?);
        let mut expander = library
            .expander
            .with_registry(registry)
            .map_err(infeasible)?
            .with_source_factory(factory)
            .with_source_leaf_limit(leaf_limit);
        expander
            .expand_bound(
                "oracle_call",
                &[
                    Value::Str("$root".into()),
                    Value::Arr(self.arguments.clone()),
                ],
                bundles,
                &[bennett_isa_conformance::ref_language::SourceSymbol {
                    name: "$root".into(),
                    hash: self.closure_hash.clone(),
                    oracle_invocation: Some(self.context.clone()),
                    adjoint: false,
                }],
            )
            .map_err(infeasible)
    }
    pub fn object(&self) -> &'a Object {
        self.object
    }
    pub fn function(&self) -> &'a crate::Function {
        self.function
    }
    pub fn closure_hash(&self) -> &str {
        &self.closure_hash
    }
    pub fn budget(&self) -> crate::Budget {
        self.budget
    }
    pub fn context(&self) -> &Value {
        &self.context
    }
    pub fn arguments(&self) -> &[Value] {
        &self.arguments
    }

    /// Invoke a source-body builder afresh for THIS capture. The builder must
    /// derive oracle.construct inputs from this object's source/layout and
    /// budget; its output is a constructor body, never a replacement catalog.
    /// A hash-only registry is built only after this invocation is specialised.
    /// Nested source calls must use their own captures. This is the explicit
    /// builder boundary, not a general implementation of source lowering.
    pub fn expand(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
        builder: impl FnOnce(&Self) -> Result<Value>,
    ) -> Result<Vec<Leaf>> {
        let parameters = source_parameters(self.object, self.function)?;
        let entry = |body| {
            Value::Obj(std::collections::BTreeMap::from([
                ("parameters".into(), Value::Arr(parameters.clone())),
                ("body".into(), body),
            ]))
        };
        library.expand(
            "oracle_call",
            &[
                Value::Str(self.closure_hash.clone()),
                Value::Arr(self.arguments.clone()),
            ],
            bundles,
            Registry::from([(self.closure_hash.clone(), entry(Value::Arr(Vec::new())))]),
        )?;
        let body = builder(self)?;
        let registry = Registry::from([(
            self.closure_hash.clone(),
            Value::Obj(std::collections::BTreeMap::from([
                ("parameters".into(), Value::Arr(parameters)),
                ("body".into(), body),
            ])),
        )]);
        library.expand(
            "oracle_call",
            &[
                Value::Str(self.closure_hash.clone()),
                Value::Arr(self.arguments.clone()),
            ],
            bundles,
            registry,
        )
    }
}

pub(crate) fn source_parameters(object: &Object, function: &crate::Function) -> Result<Vec<Value>> {
    let mut parameters = Vec::new();
    for p in function.params.iter().flatten() {
        let mut formal =
            std::collections::BTreeMap::from([("name".into(), Value::Str(p.name.clone()))]);
        match &p.ty {
            crate::Type::Wire(w) | crate::Type::Reg(w) => {
                formal.insert("type".into(), Value::Str("wire".into()));
                // Conformance scalar expressions use the exact JCS subset.
                if *w >= 1u64 << 53 {
                    return Err(crate::resource_error(
                        "§11.1",
                        "constructor width exceeds exact scalar range",
                    ));
                }
                formal.insert("width".into(), Value::Num(*w as f64));
            }
            crate::Type::Region(a, w) => {
                if *a >= 1u64 << 53 || *w >= 1u64 << 53 {
                    return Err(crate::resource_error(
                        "§11.1",
                        "constructor region exceeds exact scalar range",
                    ));
                }
                formal.insert("type".into(), Value::Str("region".into()));
                formal.insert("address".into(), Value::Num(*a as f64));
                formal.insert("cell".into(), Value::Num(*w as f64));
                formal.insert(
                    "role".into(),
                    Value::Str(p.role.clone().unwrap_or_else(|| "in".into())),
                );
            }
            ty => {
                formal.insert("type".into(), Value::Str("value".into()));
                let ty = if *ty == crate::Type::Ptr {
                    Value::Obj(std::collections::BTreeMap::from([(
                        "ptr".into(),
                        Value::Num(object.ptr_width as f64),
                    )]))
                } else {
                    Value::Str(ty.text())
                };
                formal.insert("isa_type".into(), ty);
            }
        }
        parameters.push(Value::Obj(formal));
    }
    Ok(parameters)
}
