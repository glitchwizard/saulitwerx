//! Saulitwerx document model.
//!
//! This crate is deliberately kernel-agnostic: it owns the *history* of a part
//! (an ordered, editable feature tree with named parameters) and knows which
//! features must be rebuilt when something changes. Geometry evaluation lives
//! in a separate kernel crate (OpenCascade bindings, see docs/ARCHITECTURE.md).

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
    pub fn set_param(&mut self, name: &str, value: f64) {
        match self.find_mut(name) {
            Some(p) => p.value = value,
            None => self.push_param(name.to_owned(), ParamKind::User, value, ""),
        }
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
    pub fn dirty_after_param_change(&self, param: &str) -> Vec<FeatureId> {
        let seeds = self
            .history
            .iter()
            .filter(|f| f.kind.params().contains(&param))
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
}
