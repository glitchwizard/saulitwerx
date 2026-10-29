//! Saulitwerx document model.
//!
//! This crate is deliberately kernel-agnostic: it owns the *history* of a part
//! (an ordered, editable feature tree with named parameters) and knows which
//! features must be rebuilt when something changes. Geometry evaluation lives
//! in a separate kernel crate (OpenCascade bindings, see docs/ARCHITECTURE.md).

pub mod expr;

use expr::{Dim, Expr, ExprError, ExprErrorKind, Quantity, Unit};
use std::collections::BTreeSet;
use std::fmt;

/// Stable identifier of a feature within a [`Document`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FeatureId(pub u32);

/// The kinds of feature the modeler understands. Grows milestone by milestone
/// (see docs/ROADMAP.md).
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureKind {
    /// A 2D sketch on a plane. Contents are owned by the sketch crate.
    Sketch,
    /// Linear extrusion of a sketch profile by `depth` (parameter name).
    Extrude { profile: FeatureId, depth: String },
    /// Revolution of a sketch profile by `angle_deg` (parameter name).
    Revolve {
        profile: FeatureId,
        angle_deg: String,
    },
}

impl FeatureKind {
    /// Features this one consumes as inputs.
    pub fn inputs(&self) -> Vec<FeatureId> {
        match self {
            FeatureKind::Sketch => vec![],
            FeatureKind::Extrude { profile, .. } | FeatureKind::Revolve { profile, .. } => {
                vec![*profile]
            }
        }
    }

    /// Parameter names this feature reads.
    pub fn params(&self) -> Vec<&str> {
        match self {
            FeatureKind::Sketch => vec![],
            FeatureKind::Extrude { depth, .. } => vec![depth.as_str()],
            FeatureKind::Revolve { angle_deg, .. } => vec![angle_deg.as_str()],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Feature {
    pub id: FeatureId,
    pub name: String,
    pub kind: FeatureKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    UnknownFeature(FeatureId),
    UnknownParameter(String),
    /// A parameter with this name already exists.
    DuplicateParameter(String),
    /// The name does not match `[A-Za-z_][A-Za-z0-9_]*`.
    InvalidParameterName(String),
    /// An input must appear earlier in the history than the feature using it.
    ForwardReference {
        feature: FeatureId,
        input: FeatureId,
    },
    /// The expression for `param` doesn't parse, names an unknown parameter,
    /// or fails to evaluate.
    Expr {
        param: String,
        error: ExprError,
    },
    /// Parameter expressions would depend on each other in a loop; the names
    /// on the cycle, in dependency order.
    ParameterCycle(Vec<String>),
    /// The expression's dimension doesn't match the parameter's unit.
    ParameterDimension {
        param: String,
        expected: Dim,
        found: Dim,
    },
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::UnknownFeature(id) => write!(f, "unknown feature {}", id.0),
            ModelError::UnknownParameter(p) => write!(f, "unknown parameter `{p}`"),
            ModelError::DuplicateParameter(p) => {
                write!(f, "a parameter named `{p}` already exists")
            }
            ModelError::InvalidParameterName(p) => write!(
                f,
                "invalid parameter name `{p}`: must start with a letter or `_` \
                 and contain only letters, digits and `_`"
            ),
            ModelError::ForwardReference { feature, input } => write!(
                f,
                "feature {} references feature {} which is not earlier in the history",
                feature.0, input.0
            ),
            ModelError::Expr { param, error } => {
                write!(f, "expression for `{param}`: {error}")
            }
            ModelError::ParameterCycle(names) => {
                let first = names.first().map(String::as_str).unwrap_or("");
                write!(
                    f,
                    "parameter expressions form a cycle: {} -> {first}",
                    names.join(" -> ")
                )
            }
            ModelError::ParameterDimension {
                param,
                expected,
                found,
            } => write!(
                f,
                "expression for `{param}` gives {found}, but its unit expects {expected}"
            ),
        }
    }
}

impl std::error::Error for ModelError {}

/// Where a [`Parameter`] came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    /// Created by a sketch or feature dimension; auto-named `d0`, `d1`, ….
    Model,
    /// Created explicitly by the user.
    User,
}

/// One row of a document's parameter table.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// Unique name, matching `[A-Za-z_][A-Za-z0-9_]*` (see [`is_valid_param_name`]).
    pub name: String,
    pub kind: ParamKind,
    pub value: f64,
    /// Free-text unit such as `"mm"` or `"deg"`; may be empty.
    pub unit: String,
    pub comment: String,
    /// The expression source the user typed (see [`Document::set_param_expr`]),
    /// or `None` for a plain value.
    pub expr: Option<String>,
}

/// Whether `name` is a valid parameter name: `[A-Za-z_][A-Za-z0-9_]*` (ASCII only).
pub fn is_valid_param_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A part document: a parameter table plus an ordered feature history.
#[derive(Debug, Default, Clone)]
pub struct Document {
    /// Parameters in creation order.
    params: Vec<Parameter>,
    /// Next candidate `N` for an auto-named model parameter `d<N>`. Only grows,
    /// so a model name is never handed out twice.
    next_model_index: u64,
    history: Vec<Feature>,
    next_id: u32,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the value of parameter `name`. An unknown name creates a
    /// [`ParamKind::User`] parameter with an empty unit (the name is not validated,
    /// to keep existing callers working; prefer [`Document::add_user_param`]).
    ///
    /// A parameter that had an expression keeps only the plain value. Every
    /// parameter whose expression depends on `name` is re-evaluated; if that
    /// fails (e.g. a division by zero), those dependents keep their old values.
    pub fn set_param(&mut self, name: &str, value: f64) {
        match self.find_mut(name) {
            Some(p) => {
                p.value = value;
                p.expr = None;
                let mut params = self.params.clone();
                if evaluate(&mut params).is_ok() {
                    self.params = params;
                }
            }
            None => self.push_param(name.to_owned(), ParamKind::User, value, ""),
        }
    }

    /// Give parameter `name` the expression `src` (e.g. `d0 / 2 + 5 mm`) and
    /// re-evaluate the table. A unitless result is taken in the parameter's
    /// own unit. On any error the document is left exactly as it was.
    pub fn set_param_expr(&mut self, name: &str, src: &str) -> Result<(), ModelError> {
        let i = self
            .params
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| ModelError::UnknownParameter(name.to_owned()))?;
        let expr_err = |error| ModelError::Expr {
            param: name.to_owned(),
            error,
        };
        let e = expr::parse(src).map_err(expr_err)?;
        for (n, offset) in name_refs(&e) {
            if self.find(&n).is_none() {
                return Err(expr_err(ExprError {
                    kind: ExprErrorKind::UnknownName(n),
                    offset,
                }));
            }
        }
        let mut params = self.params.clone();
        params[i].expr = Some(src.to_owned());
        evaluate(&mut params)?;
        self.params = params;
        Ok(())
    }

    /// Every parameter name, each after the parameters its expression names,
    /// ties broken by creation order (see [`dependency_order`]).
    pub fn evaluation_order(&self) -> Vec<String> {
        let (mut order, rest) = dependency_order(&self.params);
        order.extend(rest);
        order
            .into_iter()
            .map(|i| self.params[i].name.clone())
            .collect()
    }

    pub fn param(&self, name: &str) -> Option<f64> {
        self.find(name).map(|p| p.value)
    }

    /// All parameters, in creation order.
    pub fn parameters(&self) -> &[Parameter] {
        &self.params
    }

    /// Create a [`ParamKind::Model`] parameter named `d<N>` and return its name.
    /// `N` starts at 0 and only increases; names already taken are skipped.
    pub fn add_model_param(&mut self, value: f64, unit: &str) -> String {
        loop {
            let name = format!("d{}", self.next_model_index);
            self.next_model_index += 1;
            if self.find(&name).is_none() {
                self.push_param(name.clone(), ParamKind::Model, value, unit);
                return name;
            }
        }
    }

    /// Create a [`ParamKind::User`] parameter.
    pub fn add_user_param(&mut self, name: &str, value: f64, unit: &str) -> Result<(), ModelError> {
        self.check_new_name(name)?;
        self.push_param(name.to_owned(), ParamKind::User, value, unit);
        Ok(())
    }

    /// Rename parameter `old` to `new` and update every feature that reads it.
    /// Renaming a parameter to its own name is a no-op.
    pub fn rename_param(&mut self, old: &str, new: &str) -> Result<(), ModelError> {
        if self.find(old).is_none() {
            return Err(ModelError::UnknownParameter(old.to_owned()));
        }
        if old == new {
            return Ok(());
        }
        self.check_new_name(new)?;
        if let Some(p) = self.find_mut(old) {
            p.name = new.to_owned();
        }
        for p in &mut self.params {
            if let Some(src) = &mut p.expr {
                *src = rename_in_expr(src, old, new);
            }
        }
        for f in &mut self.history {
            match &mut f.kind {
                FeatureKind::Sketch => {}
                FeatureKind::Extrude { depth: r, .. }
                | FeatureKind::Revolve { angle_deg: r, .. } => {
                    if r == old {
                        *r = new.to_owned();
                    }
                }
            }
        }
        Ok(())
    }

    /// Set the free-text comment of parameter `name`.
    pub fn set_param_comment(&mut self, name: &str, comment: &str) -> Result<(), ModelError> {
        let p = self
            .find_mut(name)
            .ok_or_else(|| ModelError::UnknownParameter(name.to_owned()))?;
        p.comment = comment.to_owned();
        Ok(())
    }

    fn find(&self, name: &str) -> Option<&Parameter> {
        self.params.iter().find(|p| p.name == name)
    }

    fn find_mut(&mut self, name: &str) -> Option<&mut Parameter> {
        self.params.iter_mut().find(|p| p.name == name)
    }

    fn check_new_name(&self, name: &str) -> Result<(), ModelError> {
        if !is_valid_param_name(name) {
            return Err(ModelError::InvalidParameterName(name.to_owned()));
        }
        if self.find(name).is_some() {
            return Err(ModelError::DuplicateParameter(name.to_owned()));
        }
        Ok(())
    }

    fn push_param(&mut self, name: String, kind: ParamKind, value: f64, unit: &str) {
        self.params.push(Parameter {
            name,
            kind,
            value,
            unit: unit.to_owned(),
            comment: String::new(),
            expr: None,
        });
    }

    pub fn history(&self) -> &[Feature] {
        &self.history
    }

    /// Append a feature to the end of the history after validating its inputs.
    pub fn add_feature(&mut self, name: &str, kind: FeatureKind) -> Result<FeatureId, ModelError> {
        let id = FeatureId(self.next_id);
        for input in kind.inputs() {
            if !self.history.iter().any(|f| f.id == input) {
                return Err(ModelError::ForwardReference { feature: id, input });
            }
        }
        for p in kind.params() {
            if self.find(p).is_none() {
                return Err(ModelError::UnknownParameter(p.to_owned()));
            }
        }
        self.next_id += 1;
        self.history.push(Feature {
            id,
            name: name.to_owned(),
            kind,
        });
        Ok(id)
    }

    /// Features that must be rebuilt, in history order, after `param` changes.
    /// Features reading a parameter whose expression depends on `param`,
    /// directly or through other expressions, count too.
    pub fn dirty_after_param_change(&self, param: &str) -> Vec<FeatureId> {
        let mut changed = vec![param.to_owned()];
        // Parameters in dependency order, so one pass reaches every dependent.
        for i in dependency_order(&self.params).0 {
            let p = &self.params[i];
            let reads_changed = p
                .expr
                .as_deref()
                .and_then(|src| expr::parse(src).ok())
                .is_some_and(|e| e.names().iter().any(|n| changed.contains(n)));
            if reads_changed && !changed.contains(&p.name) {
                changed.push(p.name.clone());
            }
        }
        let seeds = self
            .history
            .iter()
            .filter(|f| {
                f.kind
                    .params()
                    .iter()
                    .any(|n| changed.iter().any(|c| c == n))
            })
            .map(|f| f.id);
        self.propagate(seeds)
    }

    /// Features that must be rebuilt, in history order, after `feature` is edited.
    pub fn dirty_after_feature_edit(
        &self,
        feature: FeatureId,
    ) -> Result<Vec<FeatureId>, ModelError> {
        if !self.history.iter().any(|f| f.id == feature) {
            return Err(ModelError::UnknownFeature(feature));
        }
        Ok(self.propagate(std::iter::once(feature)))
    }

    fn propagate(&self, seeds: impl Iterator<Item = FeatureId>) -> Vec<FeatureId> {
        let mut dirty: BTreeSet<FeatureId> = seeds.collect();
        // History is topologically ordered by construction, so one forward pass suffices.
        let mut out = Vec::new();
        for f in &self.history {
            if dirty.contains(&f.id) || f.kind.inputs().iter().any(|i| dirty.contains(i)) {
                dirty.insert(f.id);
                out.push(f.id);
            }
        }
        out
    }
}

/// Identifiers referenced by `e`, with their byte offsets, in source order.
fn name_refs(e: &Expr) -> Vec<(String, usize)> {
    fn walk(e: &Expr, out: &mut Vec<(String, usize)>) {
        match e {
            Expr::Num(_) => {}
            Expr::Name { name, offset } => out.push((name.clone(), *offset)),
            Expr::WithUnit { inner, .. } | Expr::Neg(inner) => walk(inner, out),
            Expr::Bin { lhs, rhs, .. } => {
                walk(lhs, out);
                walk(rhs, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(e, &mut out);
    out
}

/// `src` with every identifier `old` replaced by `new`. Only whole
/// identifiers change (as the parser sees them), never unit suffixes or
/// longer names such as `d10` for `d1`.
fn rename_in_expr(src: &str, old: &str, new: &str) -> String {
    let Ok(e) = expr::parse(src) else {
        return src.to_owned();
    };
    let mut out = src.to_owned();
    let mut refs = name_refs(&e);
    refs.sort_by_key(|&(_, offset)| std::cmp::Reverse(offset));
    for (name, offset) in refs {
        if name == old {
            out.replace_range(offset..offset + old.len(), new);
        }
    }
    out
}

/// For each parameter, the indices of the parameters its expression names.
fn dependencies(params: &[Parameter]) -> Vec<Vec<usize>> {
    params
        .iter()
        .map(|p| {
            let names = p
                .expr
                .as_deref()
                .and_then(|src| expr::parse(src).ok())
                .map(|e| e.names())
                .unwrap_or_default();
            names
                .iter()
                .filter_map(|n| params.iter().position(|q| &q.name == n))
                .collect()
        })
        .collect()
}

/// Topological order of the parameter table by Kahn's algorithm (A. B. Kahn,
/// "Topological sorting of large networks", *CACM* 5(11), 1962; Cormen et
/// al., *Introduction to Algorithms*, 3rd ed., §22.4): repeatedly take the
/// earliest-created parameter whose dependencies are all placed, so ties
/// follow creation order. Returns the ordered indices and the indices left
/// over, which are non-empty only when expressions form a cycle.
fn dependency_order(params: &[Parameter]) -> (Vec<usize>, Vec<usize>) {
    let deps = dependencies(params);
    let mut waiting: Vec<usize> = deps.iter().map(Vec::len).collect();
    let mut ready: BTreeSet<usize> = (0..params.len()).filter(|&i| waiting[i] == 0).collect();
    let mut order = Vec::with_capacity(params.len());
    while let Some(i) = ready.pop_first() {
        order.push(i);
        for (j, d) in deps.iter().enumerate() {
            if d.contains(&i) {
                waiting[j] -= 1;
                if waiting[j] == 0 {
                    ready.insert(j);
                }
            }
        }
    }
    let rest = (0..params.len()).filter(|i| !order.contains(i)).collect();
    (order, rest)
}

/// A cycle among the `rest` of [`dependency_order`]. Each left-over
/// parameter depends on another left-over one, so following the first such
/// dependency from the earliest must revisit a parameter.
fn find_cycle(params: &[Parameter], rest: &[usize]) -> Vec<String> {
    let deps = dependencies(params);
    let mut path: Vec<usize> = Vec::new();
    let mut cur = rest[0];
    while !path.contains(&cur) {
        path.push(cur);
        match deps[cur].iter().find(|d| rest.contains(d)) {
            Some(&next) => cur = next,
            None => return Vec::new(),
        }
    }
    let start = path.iter().position(|&i| i == cur).unwrap_or(0);
    path[start..]
        .iter()
        .map(|&i| params[i].name.clone())
        .collect()
}

/// The dimension and base-unit factor a parameter's `unit` stands for.
/// Units that aren't length or angle units count as unitless.
fn unit_dim(p: &Parameter) -> (Dim, f64) {
    Unit::from_suffix(&p.unit).map_or((Dim::NONE, 1.0), |u| (u.dim(), u.factor()))
}

fn param_quantity(p: &Parameter) -> Quantity {
    let (dim, factor) = unit_dim(p);
    Quantity::new(p.value * factor, dim)
}

/// Re-evaluate every expression in dependency order, writing the values.
fn evaluate(params: &mut [Parameter]) -> Result<(), ModelError> {
    let (order, rest) = dependency_order(params);
    if !rest.is_empty() {
        return Err(ModelError::ParameterCycle(find_cycle(params, &rest)));
    }
    for i in order {
        let Some(src) = params[i].expr.clone() else {
            continue;
        };
        let err = |error| ModelError::Expr {
            param: params[i].name.clone(),
            error,
        };
        let e = expr::parse(&src).map_err(err)?;
        let resolve = |n: &str| params.iter().find(|p| p.name == n).map(param_quantity);
        let q = expr::eval(&e, &resolve, Unit::Mm).map_err(err)?;
        let (dim, factor) = unit_dim(&params[i]);
        params[i].value = if q.dim == dim {
            q.value / factor
        } else if q.dim == Dim::NONE {
            q.value
        } else {
            return Err(ModelError::ParameterDimension {
                param: params[i].name.clone(),
                expected: dim,
                found: q.dim,
            });
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block_and_shaft() -> (Document, FeatureId, FeatureId, FeatureId, FeatureId) {
        let mut d = Document::new();
        d.set_param("thickness", 10.0);
        d.set_param("sweep", 360.0);
        let s1 = d.add_feature("Sketch1", FeatureKind::Sketch).unwrap();
        let e1 = d
            .add_feature(
                "Extrude1",
                FeatureKind::Extrude {
                    profile: s1,
                    depth: "thickness".into(),
                },
            )
            .unwrap();
        let s2 = d.add_feature("Sketch2", FeatureKind::Sketch).unwrap();
        let r1 = d
            .add_feature(
                "Revolve1",
                FeatureKind::Revolve {
                    profile: s2,
                    angle_deg: "sweep".into(),
                },
            )
            .unwrap();
        (d, s1, e1, s2, r1)
    }

    #[test]
    fn param_change_dirties_only_consumers() {
        let (d, _s1, e1, _s2, r1) = block_and_shaft();
        assert_eq!(d.dirty_after_param_change("thickness"), vec![e1]);
        assert_eq!(d.dirty_after_param_change("sweep"), vec![r1]);
        assert!(d.dirty_after_param_change("nope").is_empty());
    }

    #[test]
    fn sketch_edit_propagates_downstream() {
        let (d, s1, e1, _s2, _r1) = block_and_shaft();
        assert_eq!(d.dirty_after_feature_edit(s1).unwrap(), vec![s1, e1]);
    }

    #[test]
    fn rejects_forward_reference_and_unknown_param() {
        let mut d = Document::new();
        d.set_param("h", 1.0);
        let err = d
            .add_feature(
                "E",
                FeatureKind::Extrude {
                    profile: FeatureId(7),
                    depth: "h".into(),
                },
            )
            .unwrap_err();
        assert!(matches!(err, ModelError::ForwardReference { .. }));
        let s = d.add_feature("S", FeatureKind::Sketch).unwrap();
        let err = d
            .add_feature(
                "E",
                FeatureKind::Extrude {
                    profile: s,
                    depth: "missing".into(),
                },
            )
            .unwrap_err();
        assert_eq!(err, ModelError::UnknownParameter("missing".into()));
    }

    #[test]
    fn model_params_are_named_in_sequence() {
        let mut d = Document::new();
        assert_eq!(d.add_model_param(1.0, "mm"), "d0");
        assert_eq!(d.add_model_param(2.0, "mm"), "d1");
        assert_eq!(d.add_model_param(3.0, "deg"), "d2");
        let p = &d.parameters()[2];
        assert_eq!(p.kind, ParamKind::Model);
        assert_eq!((p.value, p.unit.as_str()), (3.0, "deg"));
        assert_eq!(p.comment, "");
    }

    #[test]
    fn model_param_names_are_not_reused_after_rename() {
        let mut d = Document::new();
        for _ in 0..3 {
            d.add_model_param(1.0, "mm");
        }
        d.rename_param("d1", "width").unwrap();
        assert_eq!(d.add_model_param(1.0, "mm"), "d3");
        // The freed name `d1` is not handed out again.
        assert_eq!(d.add_model_param(1.0, "mm"), "d4");
    }

    #[test]
    fn model_param_naming_skips_taken_names() {
        let mut d = Document::new();
        for _ in 0..4 {
            d.add_model_param(1.0, "mm");
        }
        d.add_user_param("d4", 5.0, "").unwrap();
        assert_eq!(d.add_model_param(1.0, "mm"), "d5");
        assert_eq!(d.param("d4"), Some(5.0));
        assert_eq!(d.parameters()[4].kind, ParamKind::User);
    }

    #[test]
    fn param_name_validation() {
        for ok in ["d0", "_x", "Width2", "_", "a_b"] {
            assert!(is_valid_param_name(ok), "{ok} should be valid");
        }
        for bad in ["2a", "a-b", "", "a b", "é", "a.b"] {
            assert!(!is_valid_param_name(bad), "{bad:?} should be invalid");
            let mut d = Document::new();
            assert_eq!(
                d.add_user_param(bad, 1.0, ""),
                Err(ModelError::InvalidParameterName(bad.into()))
            );
            d.set_param("p", 1.0);
            assert_eq!(
                d.rename_param("p", bad),
                Err(ModelError::InvalidParameterName(bad.into()))
            );
            assert_eq!(d.param("p"), Some(1.0));
        }
    }

    #[test]
    fn duplicate_names_are_rejected() {
        let mut d = Document::new();
        let d0 = d.add_model_param(1.0, "mm");
        d.add_user_param("width", 2.0, "mm").unwrap();
        assert_eq!(
            d.add_user_param("width", 3.0, ""),
            Err(ModelError::DuplicateParameter("width".into()))
        );
        assert_eq!(
            d.add_user_param(&d0, 3.0, ""),
            Err(ModelError::DuplicateParameter(d0.clone()))
        );
        assert_eq!(
            d.rename_param(&d0, "width"),
            Err(ModelError::DuplicateParameter("width".into()))
        );
        assert_eq!(d.param("width"), Some(2.0));
        assert_eq!(d.param(&d0), Some(1.0));
        assert_eq!(d.parameters().len(), 2);
    }

    #[test]
    fn rename_unknown_and_comment_unknown_fail() {
        let mut d = Document::new();
        assert_eq!(
            d.rename_param("nope", "x"),
            Err(ModelError::UnknownParameter("nope".into()))
        );
        assert_eq!(
            d.set_param_comment("nope", "c"),
            Err(ModelError::UnknownParameter("nope".into()))
        );
        d.set_param("x", 1.0);
        d.set_param_comment("x", "plate thickness").unwrap();
        assert_eq!(d.parameters()[0].comment, "plate thickness");
        assert_eq!(d.rename_param("x", "x"), Ok(()));
    }

    #[test]
    fn rename_updates_feature_references_and_dirty_propagation() {
        let (mut d, _s1, _e1, _s2, _r1) = block_and_shaft();
        let thickness_dirty = d.dirty_after_param_change("thickness");
        let sweep_dirty = d.dirty_after_param_change("sweep");
        d.rename_param("thickness", "plate").unwrap();
        d.rename_param("sweep", "d0").unwrap();
        assert_eq!(d.dirty_after_param_change("plate"), thickness_dirty);
        assert_eq!(d.dirty_after_param_change("d0"), sweep_dirty);
        assert!(d.dirty_after_param_change("thickness").is_empty());
        assert!(d.dirty_after_param_change("sweep").is_empty());
        assert_eq!(d.param("plate"), Some(10.0));
        assert_eq!(d.param("thickness"), None);
        assert!(matches!(
            &d.history()[1].kind,
            FeatureKind::Extrude { depth, .. } if depth == "plate"
        ));
        // `d0` is taken by the renamed user parameter, so the next model name skips it.
        assert_eq!(d.add_model_param(1.0, "mm"), "d1");
    }

    #[test]
    fn set_param_creates_user_param_and_parameters_keep_creation_order() {
        let mut d = Document::new();
        d.set_param("zeta", 1.0);
        let m = d.add_model_param(2.0, "mm");
        d.add_user_param("alpha", 3.0, "deg").unwrap();
        d.set_param("zeta", 4.0);
        let names: Vec<_> = d.parameters().iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["zeta", m.as_str(), "alpha"]);
        let zeta = &d.parameters()[0];
        assert_eq!(zeta.kind, ParamKind::User);
        assert_eq!((zeta.value, zeta.unit.as_str()), (4.0, ""));
    }

    #[test]
    fn new_errors_have_clear_messages() {
        assert_eq!(
            ModelError::DuplicateParameter("w".into()).to_string(),
            "a parameter named `w` already exists"
        );
        let msg = ModelError::InvalidParameterName("2a".into()).to_string();
        assert!(msg.starts_with("invalid parameter name `2a`"), "{msg}");
    }

    fn close(a: Option<f64>, b: f64, tol: f64) -> bool {
        a.is_some_and(|a| (a - b).abs() <= tol)
    }

    #[test]
    fn expressions_take_the_parameter_unit() {
        let mut d = Document::new();
        let d0 = d.add_model_param(40.0, "mm");
        let d1 = d.add_model_param(0.0, "mm");
        assert!(d.parameters().iter().all(|p| p.expr.is_none()));
        d.set_param_expr(&d1, "d0 / 2 + 5 mm").unwrap();
        assert!(close(d.param(&d1), 25.0, 1e-12));
        assert_eq!(d.parameters()[1].expr.as_deref(), Some("d0 / 2 + 5 mm"));
        d.add_user_param("w", 0.0, "in").unwrap();
        d.set_param_expr("w", "1 in").unwrap();
        assert!(close(d.param("w"), 1.0, 1e-12));
        d.add_user_param("ang", 0.0, "deg").unwrap();
        d.set_param_expr("ang", "90 deg").unwrap();
        assert!(close(d.param("ang"), 90.0, 1e-9));
        // Unitless results are taken in the parameter's own unit.
        d.set_param_expr("ang", "5").unwrap();
        assert!(close(d.param("ang"), 5.0, 1e-12));
        d.set_param_expr(&d0, "5").unwrap();
        assert!(close(d.param(&d0), 5.0, 1e-12));
        assert!(close(d.param(&d1), 7.5, 1e-12));
        assert_eq!(
            d.set_param_expr(&d1, "2 deg"),
            Err(ModelError::ParameterDimension {
                param: d1.clone(),
                expected: Dim::LENGTH,
                found: Dim::ANGLE,
            })
        );
        assert!(close(d.param(&d1), 7.5, 1e-12));
    }

    #[test]
    fn evaluation_order_puts_dependencies_first() {
        let mut d = Document::new();
        for n in ["c", "b", "a"] {
            d.add_user_param(n, 0.0, "mm").unwrap();
        }
        assert_eq!(d.evaluation_order(), ["c", "b", "a"]);
        d.set_param_expr("a", "3 mm").unwrap();
        d.set_param_expr("b", "a * 2").unwrap();
        d.set_param_expr("c", "a + b").unwrap();
        assert_eq!(d.evaluation_order(), ["a", "b", "c"]);
        assert!(close(d.param("c"), 9.0, 1e-12));
    }

    #[test]
    fn cycles_are_rejected() {
        let mut d = Document::new();
        for n in ["a", "b", "c"] {
            d.add_user_param(n, 1.0, "mm").unwrap();
        }
        d.set_param_expr("a", "b + 1 mm").unwrap();
        let Err(ModelError::ParameterCycle(cycle)) = d.set_param_expr("b", "a") else {
            panic!("expected a cycle");
        };
        assert!(cycle == ["a", "b"] || cycle == ["b", "a"], "{cycle:?}");
        d.set_param_expr("b", "c").unwrap();
        let Err(ModelError::ParameterCycle(cycle)) = d.set_param_expr("c", "a * 2") else {
            panic!("expected a cycle");
        };
        assert_eq!(cycle.len(), 3, "{cycle:?}");
        let Err(ModelError::ParameterCycle(cycle)) = d.set_param_expr("c", "c + 1 mm") else {
            panic!("expected a cycle");
        };
        assert_eq!(cycle, ["c"]);
        assert!(d.parameters()[2].expr.is_none());
    }

    #[test]
    fn set_param_re_evaluates_dependents() {
        let mut d = Document::new();
        for _ in 0..3 {
            d.add_model_param(0.0, "mm");
        }
        d.set_param_expr("d0", "40 mm").unwrap();
        d.set_param_expr("d1", "d0 / 2").unwrap();
        d.set_param_expr("d2", "d1 + d0").unwrap();
        d.set_param("d0", 60.0);
        assert!(close(d.param("d1"), 30.0, 1e-12));
        assert!(close(d.param("d2"), 90.0, 1e-12));
        assert!(d.parameters()[0].expr.is_none());
        d.set_param("d1", 7.0);
        assert!(d.parameters()[1].expr.is_none());
        assert!(close(d.param("d2"), 67.0, 1e-12));
    }

    #[test]
    fn set_param_keeps_dependents_when_re_evaluation_fails() {
        let mut d = Document::new();
        d.add_user_param("n", 2.0, "").unwrap();
        d.add_user_param("w", 0.0, "mm").unwrap();
        d.set_param_expr("w", "10 mm / n").unwrap();
        d.set_param("n", 0.0);
        assert_eq!(d.param("n"), Some(0.0));
        assert!(close(d.param("w"), 5.0, 1e-12));
    }

    #[test]
    fn dirty_propagates_through_expressions() {
        let (mut d, _s1, e1, _s2, _r1) = block_and_shaft();
        d.set_param("base_t", 5.0);
        d.set_param_expr("thickness", "base_t * 2").unwrap();
        assert_eq!(d.param("thickness"), Some(10.0));
        assert_eq!(
            d.dirty_after_param_change("base_t"),
            d.dirty_after_param_change("thickness")
        );
        assert_eq!(d.dirty_after_param_change("base_t"), vec![e1]);
    }

    #[test]
    fn rename_rewrites_whole_identifiers_in_expressions() {
        let mut d = Document::new();
        for i in 0..11 {
            d.add_model_param(i as f64 + 1.0, "mm");
        }
        d.add_user_param("in", 2.0, "mm").unwrap();
        d.set_param_expr("d5", "d1 / 2 + d10 + 2 in + in").unwrap();
        let before: Vec<f64> = d.parameters().iter().map(|p| p.value).collect();
        d.rename_param("d1", "width").unwrap();
        d.rename_param("in", "inch").unwrap();
        assert_eq!(
            d.parameters()[5].expr.as_deref(),
            Some("width / 2 + d10 + 2 in + inch")
        );
        let after: Vec<f64> = d.parameters().iter().map(|p| p.value).collect();
        assert_eq!(before, after);
        d.set_param("width", 4.0);
        assert!(close(d.param("d5"), 2.0 + 11.0 + 50.8 + 2.0, 1e-12));
    }

    #[test]
    fn invalid_expressions_leave_the_document_unchanged() {
        let mut d = Document::new();
        for _ in 0..2 {
            d.add_model_param(10.0, "mm");
        }
        d.add_user_param("n", 3.0, "").unwrap();
        d.set_param_expr("d1", "d0 / 2").unwrap();
        let snapshot = d.parameters().to_vec();
        let cases = [
            ("nope", "1 mm"),
            ("d0", ""),
            ("d0", "1 +"),
            ("d0", "(1 mm"),
            ("d0", "2 kg"),
            ("d0", "missing + 1 mm"),
            ("d0", "5 mm / 0"),
            ("d0", "2 deg"),
            ("d0", "d1 * d1"),
            ("d0", "d1 + 2 deg"),
            ("d0", "d1"),
            ("d1", "d1 + 1 mm"),
            ("n", "1 mm"),
            ("d0", "1 $"),
            ("d0", "))"),
        ];
        for (name, src) in cases {
            assert!(d.set_param_expr(name, src).is_err(), "{name} = {src}");
            assert_eq!(d.parameters(), &snapshot[..], "{name} = {src}");
        }
    }

    #[test]
    fn expression_errors_have_clear_messages() {
        let mut d = Document::new();
        d.add_user_param("a", 1.0, "mm").unwrap();
        let e = d.set_param_expr("a", "b + 1").unwrap_err();
        assert_eq!(
            e,
            ModelError::Expr {
                param: "a".into(),
                error: ExprError {
                    kind: ExprErrorKind::UnknownName("b".into()),
                    offset: 0,
                },
            }
        );
        assert_eq!(
            e.to_string(),
            "expression for `a`: unknown name `b` at byte 0"
        );
        assert_eq!(
            ModelError::ParameterCycle(vec!["a".into(), "b".into()]).to_string(),
            "parameter expressions form a cycle: a -> b -> a"
        );
        assert_eq!(
            ModelError::ParameterDimension {
                param: "d1".into(),
                expected: Dim::LENGTH,
                found: Dim::ANGLE,
            }
            .to_string(),
            "expression for `d1` gives angle, but its unit expects length"
        );
    }
}
