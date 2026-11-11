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

/// Frozen original snapshot, retained for explicit compatibility bindings.
pub const REF09_HASH: &str = "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197";
/// Current ISA 0.9 default, with independent qjumpr address/successor widths.
pub const REF09_R1_HASH: &str = "0be00e64de17699516a9b5bb5daf308ee0c59af7e39b6bf934bad5c628f48e7a";
pub(crate) mod tokens;
const CATALOG: &str = include_str!("../../spec/ref-0.9.json");
const CATALOG_R1: &str = include_str!("../../spec/ref-0.9-r1.json");

/// Only the published complete library can satisfy a ref@0.9 binding. Private
/// witness catalogs cannot acquire its identity by supplying its descriptor.
#[derive(Clone, Debug)]
pub struct RefLibrary {
    expander: Expander,
    hash: &'static str,
}
fn infeasible(e: impl std::fmt::Display) -> crate::Error {
    let text = e.to_string();
    if let Some(reason) =
        text.strip_prefix(bennett_isa_conformance::ref_language::HOST_RESOURCE_LIMIT_PREFIX)
    {
        crate::Error::host_resource(format!("§11.1: {reason}"))
    } else {
        crate::Error::new(format!("§11.1: E_SCHEDULE_INFEASIBLE: {text}"))
    }
}
impl RefLibrary {
    pub fn published() -> Result<Self> {
        Self::for_hash(REF09_R1_HASH)
    }
    fn for_hash(hash: &str) -> Result<Self> {
        static LIBRARY: std::sync::OnceLock<Result<RefLibrary>> = std::sync::OnceLock::new();
        static ORIGINAL: std::sync::OnceLock<Result<RefLibrary>> = std::sync::OnceLock::new();
        match hash {
            REF09_R1_HASH => LIBRARY
                .get_or_init(|| Self::bind("ref", "0.9", REF09_R1_HASH, CATALOG_R1))
                .clone(),
            REF09_HASH => ORIGINAL
                .get_or_init(|| Self::bind("ref", "0.9", REF09_HASH, CATALOG))
                .clone(),
            _ => Err(infeasible("bound expansion library is unavailable")),
        }
    }
    pub fn bind(name: &str, version: &str, hash: &str, bytes: &str) -> Result<Self> {
        if (name, version) != ("ref", "0.9") {
            return Err(infeasible("bound expansion library is unavailable"));
        }
        let hash = match hash {
            REF09_HASH => REF09_HASH,
            REF09_R1_HASH => REF09_R1_HASH,
            _ => return Err(infeasible("bound expansion library is unavailable")),
        };
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
        Ok(Self { expander, hash })
    }
    pub fn hash(&self) -> &'static str {
        self.hash
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
pub(crate) struct SourceRootRequest<'a> {
    pub budget: crate::Budget,
    pub variant: &'a str,
    pub controls: &'a [bennett_isa_conformance::ref_language::Wire],
}
#[derive(Default)]
struct RootSourceContext<'a> {
    controls: &'a [bennett_isa_conformance::ref_language::Wire],
    binding: Option<bennett_isa_conformance::ref_language::SourceRootBinding>,
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
        self.expand_action_source_bound(
            name,
            args,
            bundles,
            adjoint,
            None,
            RootSourceContext::default(),
        )
        .map(|(leaves, _)| leaves)
    }
    /// Tuple-only token parameters require their canonical equality partition.
    pub fn expand_action_source_with_token_aliases(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
        aliases: &[u64],
    ) -> Result<Vec<Leaf>> {
        self.expand_action_source_bound(
            name,
            args,
            bundles,
            adjoint,
            Some(aliases),
            RootSourceContext::default(),
        )
        .map(|(leaves, _)| leaves)
    }
    /// The complete source invocation inventory, independent of emitted leaves.
    pub fn source_action_invocations(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
        aliases: Option<&[u64]>,
    ) -> Result<Vec<ResolvedSourceInvocation>> {
        let (leaves, records) = self.expand_action_source_bound(
            name,
            args,
            bundles,
            adjoint,
            aliases,
            RootSourceContext::default(),
        )?;
        resolve_source_invocations(&records, &leaves)
    }
    pub fn source_action_invocations_with_controls(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
        aliases: Option<&[u64]>,
        controls: &[bennett_isa_conformance::ref_language::Wire],
    ) -> Result<Vec<ResolvedSourceInvocation>> {
        let (leaves, records) = self.expand_action_source_bound(
            name,
            args,
            bundles,
            adjoint,
            aliases,
            RootSourceContext {
                controls,
                binding: None,
            },
        )?;
        resolve_source_invocations(&records, &leaves)
    }
    pub(crate) fn source_action_invocations_at(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        aliases: &[u64],
        request: SourceRootRequest<'_>,
    ) -> Result<Vec<ResolvedSourceInvocation>> {
        let budget = request.budget;
        let root = bennett_isa_conformance::ref_language::SourceRootBinding {
            budget: [
                budget.steps as u128,
                budget.heap,
                budget.stack,
                budget.depth,
            ],
            variant: request.variant.into(),
        };
        let (leaves, records) = self.expand_action_source_bound(
            name,
            args,
            bundles,
            false,
            Some(aliases),
            RootSourceContext {
                controls: request.controls,
                binding: Some(root),
            },
        )?;
        resolve_source_invocations(&records, &leaves)
    }
    fn expand_action_source_bound(
        &self,
        name: &str,
        args: &[Value],
        bundles: &[(&str, usize)],
        adjoint: bool,
        aliases: Option<&[u64]>,
        context: RootSourceContext<'_>,
    ) -> Result<(
        Vec<Leaf>,
        Vec<bennett_isa_conformance::ref_language::SourceInvocationRecord>,
    )> {
        self.ensure_linked()?;
        self.check_hashes()?;
        let f = self
            .functions
            .get(name)
            .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
            .ok_or_else(|| crate::Error::new("§11.1: source action is not defined".into()))?;
        let tokens = tokens::SourceTokens::bind(self, f, args, None, aliases)?;
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
        let factory = std::sync::Arc::new(crate::source_constructor::LinkedSource::new(
            self.clone(),
            tokens.clone(),
            library.hash(),
        )?);
        let mut e = library
            .expander
            .with_registry(registry)
            .map_err(infeasible)?
            .with_source_factory(factory)
            .with_source_controls(context.controls.to_vec());
        if let Some(binding) = context.binding {
            e = e.with_source_root_binding(binding);
        }
        let leaves = e
            .expand_bound(
                "action_call",
                &[Value::Str("$root".into()), Value::Arr(args.to_vec())],
                bundles,
                &[bennett_isa_conformance::ref_language::SourceSymbol {
                    name: "$root".into(),
                    hash,
                    oracle_invocation: None,
                    tokens: Some(tokens.context),
                    adjoint,
                }],
            )
            .map_err(infeasible)?;
        Ok((leaves, e.source_invocations().to_vec()))
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
            hash: Some(REF09_R1_HASH),
        });
        if (binding.name, binding.version) != ("ref", "0.9") {
            return Err(infeasible("bound expansion library is unavailable"));
        }
        RefLibrary::for_hash(
            binding
                .hash
                .ok_or_else(|| infeasible("missing expansion library hash"))?,
        )
    }
    pub(crate) fn require_ref_library_binding(&self) -> Result<()> {
        self.bound_ref_library().map(|_| ())
    }
}

/// A source-bound K77 channel, captured with its positional actual vector.
/// Cloning/forwarding this value keeps its tuple; expansion has no ambient
/// current-token state for a nested caller to overwrite. It intentionally
/// keeps opaque identities in a separate source channel, never in the tuple
/// JSON or signature arguments.
#[derive(Clone, Debug)]
pub struct OracleInvocation<'a> {
    object: &'a Object,
    function: &'a crate::Function,
    closure_hash: String,
    budget: crate::Budget,
    context: Value,
    arguments: Vec<Value>,
    pub(crate) tokens: tokens::SourceTokens,
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
        Self::capture_bound(object, name, context, source_budget, arguments, None, None)
    }
    pub fn capture_with_token_aliases(
        object: &'a Object,
        name: &str,
        context: Option<&Value>,
        source_budget: crate::Budget,
        arguments: &[Value],
        aliases: &[u64],
    ) -> Result<Self> {
        Self::capture_bound(
            object,
            name,
            context,
            source_budget,
            arguments,
            Some(aliases),
            None,
        )
    }
    pub(crate) fn capture_bound(
        object: &'a Object,
        name: &str,
        context: Option<&Value>,
        source_budget: crate::Budget,
        arguments: &[Value],
        aliases: Option<&[u64]>,
        resolved: Option<tokens::SourceTokens>,
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
        let tokens = match resolved {
            Some(tokens) => tokens,
            None => tokens::SourceTokens::bind(
                object,
                function,
                arguments,
                Some(source_budget),
                aliases,
            )?,
        };
        Ok(Self {
            object,
            function,
            closure_hash,
            budget: source_budget,
            context: context.clone(),
            arguments: arguments.to_vec(),
            tokens,
        })
    }
    /// Construct and expand this checked linked source invocation. No supplied
    /// callback or basis-input witness participates in source construction.
    pub fn expand_source(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
    ) -> Result<Vec<Leaf>> {
        self.expand_source_with_leaf_limit(
            library,
            bundles,
            bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT,
        )
    }
    /// Use a caller-selected materialisation capacity. This changes host
    /// capacity only; the source token and its ISA fault boundaries are intact.
    pub fn expand_source_with_leaf_limit(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
        leaf_limit: usize,
    ) -> Result<Vec<Leaf>> {
        self.expand_source_observed(library, bundles, leaf_limit, &[], None)
            .map(|(leaves, _)| leaves)
    }
    pub fn source_invocations(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
        controls: &[bennett_isa_conformance::ref_language::Wire],
    ) -> Result<Vec<ResolvedSourceInvocation>> {
        let variant = self
            .object
            .section("schedule")
            .and_then(|s| s.get("bindings").ok())
            .filter(|b| {
                b.get("closure_hash").and_then(crate::json::Json::string)
                    == Ok(self.closure_hash.as_str())
            })
            .map(|b| b.get("variant").and_then(crate::json::Json::string))
            .transpose()?;
        let (leaves, records) = self.expand_source_observed(
            library,
            bundles,
            bennett_isa_conformance::ref_language::DEFAULT_SOURCE_LEAF_LIMIT,
            controls,
            variant,
        )?;
        resolve_source_invocations(&records, &leaves)
    }
    fn expand_source_observed(
        &self,
        library: &RefLibrary,
        bundles: &[(&str, usize)],
        leaf_limit: usize,
        controls: &[bennett_isa_conformance::ref_language::Wire],
        variant: Option<&str>,
    ) -> Result<(
        Vec<Leaf>,
        Vec<bennett_isa_conformance::ref_language::SourceInvocationRecord>,
    )> {
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
            self.tokens.clone(),
            library.hash(),
        )?);
        let mut expander = library
            .expander
            .with_registry(registry)
            .map_err(infeasible)?
            .with_source_factory(factory)
            .with_source_leaf_limit(leaf_limit)
            .with_source_controls(controls.to_vec());
        if let Some(variant) = variant {
            expander = expander.with_source_root_binding(
                bennett_isa_conformance::ref_language::SourceRootBinding {
                    budget: [
                        self.budget.steps as u128,
                        self.budget.heap,
                        self.budget.stack,
                        self.budget.depth,
                    ],
                    variant: variant.into(),
                },
            );
        }
        let leaves = expander
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
                    tokens: Some(self.tokens.context.clone()),
                    adjoint: false,
                }],
            )
            .map_err(infeasible)?;
        Ok((leaves, expander.source_invocations().to_vec()))
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

/// A concrete semantic invocation in a source-derived construction. Its context
/// key is fully resolved and contains no producer wildcard or storage address.
#[derive(Clone, Debug)]
pub struct ResolvedSourceInvocation {
    /// Source-derived local allocation classes; `any` needs target realisation.
    pub allocation_classes: std::sync::Arc<
        std::collections::BTreeMap<bennett_isa_conformance::ref_language::Wire, String>,
    >,
    pub context_key: crate::json::Json,
    pub closure_hash: String,
    pub binding: bennett_isa_conformance::ref_language::SourceBinding,
    pub values: std::collections::BTreeMap<String, Value>,
    pub wires: std::collections::BTreeMap<String, Vec<bennett_isa_conformance::ref_language::Wire>>,
}
// Allocation identities/classes come from the independent expansion, not a
// producer frame. Retain them even for empty child bodies and inverse visits.
fn resolve_source_invocations(
    records: &[bennett_isa_conformance::ref_language::SourceInvocationRecord],
    leaves: &[Leaf],
) -> Result<Vec<ResolvedSourceInvocation>> {
    let mut classes = std::collections::BTreeMap::new();
    for leaf in leaves {
        if let Some((allocation, width, class, _)) = &leaf.allocation {
            for bit in 0..*width {
                if classes
                    .insert((allocation.clone(), bit), class.clone())
                    .is_some_and(|previous| previous != *class)
                {
                    return err("§9.4", "allocation identity changes class");
                }
            }
        }
    }
    let classes = std::sync::Arc::new(classes);
    records
        .iter()
        .map(|record| {
            let mut invocation = resolve_source_invocation(record)?;
            invocation.allocation_classes = classes.clone();
            Ok(invocation)
        })
        .collect()
}
fn resolve_source_invocation(
    record: &bennett_isa_conformance::ref_language::SourceInvocationRecord,
) -> Result<ResolvedSourceInvocation> {
    use crate::json::Json;
    use std::collections::BTreeMap;
    let invocation = record.invocations.last().ok_or_else(|| {
        crate::Error::new("§12.1: source invocation observation has no owner".into())
    })?;
    let binding = invocation
        .binding
        .clone()
        .ok_or_else(|| crate::Error::new("§12.1: unresolved source invocation binding".into()))?;
    let mut edges = Vec::new();
    for (i, edge) in record.path.iter().enumerate() {
        let array =
            |values: &[usize]| Json::Array(values.iter().map(|v| source_index(*v)).collect());
        let mut fields = BTreeMap::from([
            ("symbol".into(), Json::String(edge.point.symbol.clone())),
            ("point".into(), Json::String(edge.point.point.clone())),
            ("iteration".into(), array(&edge.point.iteration)),
            ("instance".into(), array(&edge.point.instance)),
            ("call_index".into(), source_index(edge.call_index)),
        ]);
        let owner = record
            .invocations
            .get(i)
            .ok_or_else(|| crate::Error::new("§12.1: source context ancestry mismatch".into()))?;
        if let Some(step) = record
            .steps
            .iter()
            .find(|step| step.invocation == owner.invocation)
        {
            fields.insert("step".into(), source_index(step.step));
            fields.insert(
                "phase".into(),
                Json::String(if step.reverse { "rev" } else { "fwd" }.into()),
            );
        }
        edges.push(Json::Object(fields));
    }
    Ok(ResolvedSourceInvocation {
        allocation_classes: Default::default(),
        values: record.values.clone(),
        context_key: Json::Array(edges),
        closure_hash: invocation.symbol.clone(),
        binding,
        wires: record.wires.clone(),
    })
}

mod frames;
pub(crate) use frames::validate_context_key;
pub use frames::SelectedInvocationFrame;

fn source_index(value: usize) -> crate::json::Json {
    // §9.1 exact metadata integer encoding.
    const MAX_JSON_INTEGER: u128 = 9_007_199_254_740_991;
    if value as u128 <= MAX_JSON_INTEGER {
        crate::json::Json::Number(value.to_string())
    } else {
        crate::json::Json::String(value.to_string())
    }
}
