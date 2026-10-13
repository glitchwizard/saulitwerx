//! Saulitwerx document model.
//!
//! This crate is deliberately kernel-agnostic: it owns the *history* of a part
//! (an ordered, editable feature tree with named parameters) and knows which
//! features must be rebuilt when something changes. Geometry evaluation lives
//! in a separate kernel crate (OpenCascade bindings, see docs/ARCHITECTURE.md).

use std::collections::{BTreeMap, BTreeSet};
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
            ModelError::ForwardReference { feature, input } => write!(
                f,
                "feature {} references feature {} which is not earlier in the history",
                feature.0, input.0
            ),
        }
    }
}

impl std::error::Error for ModelError {}

/// A part document: named parameters plus an ordered feature history.
#[derive(Debug, Default, Clone)]
pub struct Document {
    params: BTreeMap<String, f64>,
    history: Vec<Feature>,
    next_id: u32,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_param(&mut self, name: &str, value: f64) {
        self.params.insert(name.to_owned(), value);
    }

    pub fn param(&self, name: &str) -> Option<f64> {
        self.params.get(name).copied()
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
            if !self.params.contains_key(p) {
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
}
