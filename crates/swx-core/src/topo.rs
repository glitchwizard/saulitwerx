//! Persistent names for faces and edges (topological naming v1).
//!
//! A face is named by the feature that created it and its role in that
//! feature, never by a kernel index, so the name survives rebuilds that keep
//! the topology. A face that a later feature splits becomes a *fragment*,
//! told apart by the faces that bound it, and an edge is named by the two
//! faces it separates. This is history-based naming after Kripac (1997),
//! "A mechanism for persistently naming topological entities in history-based
//! parametric solid models", with fragments identified by topological context
//! as in Capoyleas, Chen and Hoffmann (1996), "Generic naming in generative,
//! constraint-based design". Design: `docs/specs/M3-04.md`.
//!
//! These are plain data: this module knows nothing about the kernel or the
//! sketch crate. Resolving a name to a kernel entity happens elsewhere.

use crate::{Document, FeatureId};
use std::collections::BTreeSet;
use std::fmt;

/// A sketch entity, by its sketch feature and its `swx_sketch::GeomId` value.
/// Kept as a plain `u32` so `swx-core` does not depend on `swx-sketch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityRef {
    /// The sketch feature that holds the entity.
    pub sketch: FeatureId,
    /// The entity's ID within that sketch.
    pub entity: u32,
}

/// Which face of a feature's result a name means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// The cap on the profile's plane (where an extrude starts).
    StartCap,
    /// The cap at the far end.
    EndCap,
    /// The face swept by one sketch entity.
    Side(EntityRef),
}

/// The persistent name of a face.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FaceName {
    /// Created by `feature` in `role`.
    Generated {
        /// The feature that created the face.
        feature: FeatureId,
        /// The face's role in that feature.
        role: Role,
    },
    /// One piece of `of`, left after `split_by` split it; chosen by the faces
    /// it touches. Build it with [`FaceName::fragment`] to keep it canonical.
    Fragment {
        /// The face that was split.
        of: Box<FaceName>,
        /// The feature that split it.
        split_by: FeatureId,
        /// The faces that bound this piece, sorted and without duplicates.
        bounded_by: Vec<FaceName>,
    },
}

impl FaceName {
    /// A canonical fragment: `bounded_by` (and any fragment nested in `of` or
    /// `bounded_by`) is sorted and de-duplicated.
    pub fn fragment(of: FaceName, split_by: FeatureId, bounded_by: Vec<FaceName>) -> FaceName {
        FaceName::Fragment {
            of: Box::new(of),
            split_by,
            bounded_by,
        }
        .canonical()
    }

    /// This name in canonical form, so that equal faces compare equal no
    /// matter the order their bounding faces were listed in.
    pub fn canonical(self) -> FaceName {
        match self {
            FaceName::Generated { .. } => self,
            FaceName::Fragment {
                of,
                split_by,
                bounded_by,
            } => FaceName::Fragment {
                of: Box::new(of.canonical()),
                split_by,
                bounded_by: canonical_set(bounded_by),
            },
        }
    }

    fn collect_features(&self, out: &mut BTreeSet<FeatureId>) {
        match self {
            FaceName::Generated { feature, role } => {
                out.insert(*feature);
                if let Role::Side(e) = role {
                    out.insert(e.sketch);
                }
            }
            FaceName::Fragment {
                of,
                split_by,
                bounded_by,
            } => {
                of.collect_features(out);
                out.insert(*split_by);
                for f in bounded_by {
                    f.collect_features(out);
                }
            }
        }
    }

    fn describe(&self, doc: &Document) -> String {
        match self {
            FaceName::Generated { feature, role } => {
                let role = match role {
                    Role::StartCap => "start cap".to_owned(),
                    Role::EndCap => "end cap".to_owned(),
                    Role::Side(e) => format!("side (e{})", e.entity),
                };
                format!("{} {role}", feature_name(doc, *feature))
            }
            FaceName::Fragment {
                of,
                split_by,
                bounded_by,
            } => {
                let mut s = format!(
                    "{}, piece split by {}",
                    of.describe(doc),
                    feature_name(doc, *split_by)
                );
                if !bounded_by.is_empty() {
                    s.push_str(" next to ");
                    s.push_str(&join(bounded_by, doc));
                }
                s
            }
        }
    }
}

/// The edge between two faces, plus the faces its end vertices touch when the
/// pair shares several edges. Build it with [`EdgeName::new`] to keep it canonical.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeName {
    /// The two faces, sorted.
    pub faces: [FaceName; 2],
    /// Faces touched by the edge's ends, sorted and without duplicates.
    pub ends: Vec<FaceName>,
}

impl EdgeName {
    /// The canonical edge between `a` and `b`: the faces are ordered and
    /// `ends` is sorted and de-duplicated, so `new(a, b, x) == new(b, a, x)`
    /// and the order of `ends` doesn't matter.
    pub fn new(a: FaceName, b: FaceName, ends: Vec<FaceName>) -> EdgeName {
        let (a, b) = (a.canonical(), b.canonical());
        let faces = if a <= b { [a, b] } else { [b, a] };
        EdgeName {
            faces,
            ends: canonical_set(ends),
        }
    }
}

/// A persistent reference to a face or an edge.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TopoRef {
    /// A face.
    Face(FaceName),
    /// An edge.
    Edge(EdgeName),
}

impl TopoRef {
    /// Every feature this reference names: generating features, the sketch of
    /// each side's entity, splitting features, and everything inside
    /// `bounded_by` and `ends`, recursively.
    pub fn features(&self) -> BTreeSet<FeatureId> {
        let mut out = BTreeSet::new();
        match self {
            TopoRef::Face(f) => f.collect_features(&mut out),
            TopoRef::Edge(e) => {
                for f in e.faces.iter().chain(&e.ends) {
                    f.collect_features(&mut out);
                }
            }
        }
        out
    }

    /// The reference in words, with feature names from `doc`, e.g.
    /// `face Extrusion1 end cap`. A feature missing from `doc` is written
    /// `feature #<id>`.
    pub fn describe(&self, doc: &Document) -> String {
        match self {
            TopoRef::Face(f) => format!("face {}", f.describe(doc)),
            TopoRef::Edge(e) => {
                let mut s = format!(
                    "edge between {} and {}",
                    e.faces[0].describe(doc),
                    e.faces[1].describe(doc)
                );
                if !e.ends.is_empty() {
                    s.push_str(", ending at ");
                    s.push_str(&join(&e.ends, doc));
                }
                s
            }
        }
    }

    fn kind_plural(&self) -> &'static str {
        match self {
            TopoRef::Face(_) => "faces",
            TopoRef::Edge(_) => "edges",
        }
    }
}

/// Why a reference could not be resolved to exactly one entity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResolveError {
    /// Nothing matches the reference.
    Missing {
        /// The reference that failed.
        reference: TopoRef,
        /// What is missing, in words for the user.
        reason: String,
    },
    /// Several entities match; the user must pick one.
    Ambiguous {
        /// The reference that failed.
        reference: TopoRef,
        /// How many entities match.
        matches: usize,
    },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::Missing { reason, .. } => write!(f, "reference is missing: {reason}"),
            ResolveError::Ambiguous { reference, matches } => write!(
                f,
                "reference matches {matches} {}; pick one",
                reference.kind_plural()
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

fn canonical_set(names: Vec<FaceName>) -> Vec<FaceName> {
    let mut names: Vec<FaceName> = names.into_iter().map(FaceName::canonical).collect();
    names.sort();
    names.dedup();
    names
}

fn feature_name(doc: &Document, id: FeatureId) -> String {
    doc.history()
        .iter()
        .find(|f| f.id == id)
        .map_or_else(|| format!("feature #{}", id.0), |f| f.name.clone())
}

fn join(names: &[FaceName], doc: &Document) -> String {
    names
        .iter()
        .map(|n| n.describe(doc))
        .collect::<Vec<_>>()
        .join(" and ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FeatureKind;

    /// Sketch1, Extrusion1, Extrusion2, Cut1, in that order.
    fn doc() -> (Document, [FeatureId; 4]) {
        let mut d = Document::new();
        d.add_user_param("H", 10.0, "mm").unwrap();
        let s = d.add_feature("Sketch1", FeatureKind::Sketch).unwrap();
        let mut extrude = |name: &str| {
            d.add_feature(
                name,
                FeatureKind::Extrude {
                    profile: s,
                    depth: "H".into(),
                },
            )
            .unwrap()
        };
        let e1 = extrude("Extrusion1");
        let e2 = extrude("Extrusion2");
        let c1 = extrude("Cut1");
        (d, [s, e1, e2, c1])
    }

    fn generated(feature: FeatureId, role: Role) -> FaceName {
        FaceName::Generated { feature, role }
    }

    fn side(feature: FeatureId, sketch: FeatureId, entity: u32) -> FaceName {
        generated(feature, Role::Side(EntityRef { sketch, entity }))
    }

    #[test]
    fn edge_name_is_canonical() {
        let (_, [s, e1, e2, c1]) = doc();
        let a = generated(e1, Role::EndCap);
        let b = side(e2, s, 1);
        let x = side(c1, s, 2);
        let y = generated(e1, Role::StartCap);
        let ab = EdgeName::new(a.clone(), b.clone(), vec![x.clone(), y.clone()]);
        assert_eq!(
            ab,
            EdgeName::new(b.clone(), a.clone(), vec![y.clone(), x.clone()])
        );
        assert_eq!(
            ab,
            EdgeName::new(b.clone(), a.clone(), vec![x.clone(), y.clone(), x.clone()])
        );
        assert_eq!(ab.faces, [a.clone(), b.clone()]);
        assert_eq!(ab.ends, vec![y.clone(), x.clone()]);
        assert_ne!(ab, EdgeName::new(a, b, vec![x]));
    }

    #[test]
    fn nested_fragments_are_canonical() {
        let (_, [s, e1, e2, c1]) = doc();
        let p = side(c1, s, 2);
        let q = side(c1, s, 3);
        let r = side(e2, s, 1);
        // A fragment literal with unsorted, repeated bounding faces.
        let raw_inner = FaceName::Fragment {
            of: Box::new(generated(e1, Role::EndCap)),
            split_by: c1,
            bounded_by: vec![q.clone(), p.clone(), q.clone()],
        };
        let inner = FaceName::fragment(generated(e1, Role::EndCap), c1, vec![p.clone(), q.clone()]);
        assert_eq!(raw_inner.clone().canonical(), inner);
        let outer_a = FaceName::fragment(raw_inner.clone(), e2, vec![r.clone(), p.clone()]);
        let outer_b = FaceName::fragment(inner.clone(), e2, vec![p.clone(), r.clone(), r.clone()]);
        assert_eq!(outer_a, outer_b);
        // Nested fragments inside an edge's faces and ends.
        assert_eq!(
            EdgeName::new(
                raw_inner.clone(),
                r.clone(),
                vec![outer_a.clone(), raw_inner.clone()]
            ),
            EdgeName::new(
                r.clone(),
                inner.clone(),
                vec![inner.clone(), outer_b.clone()]
            )
        );
    }

    #[test]
    fn features_of_an_edge_between_a_fragment_and_a_side() {
        let (_, [s, e1, e2, c1]) = doc();
        let frag = FaceName::fragment(generated(e1, Role::EndCap), c1, vec![side(c1, s, 2)]);
        let edge = TopoRef::Edge(EdgeName::new(frag, side(e2, s, 1), vec![]));
        assert_eq!(edge.features(), BTreeSet::from([s, e1, e2, c1]));
        // Ends count too, and a cap names no sketch.
        let cap_only = TopoRef::Face(generated(e2, Role::StartCap));
        assert_eq!(cap_only.features(), BTreeSet::from([e2]));
        let with_end = TopoRef::Edge(EdgeName::new(
            generated(e1, Role::EndCap),
            generated(e1, Role::StartCap),
            vec![generated(c1, Role::EndCap)],
        ));
        assert_eq!(with_end.features(), BTreeSet::from([e1, c1]));
    }

    #[test]
    fn describe_names_features_from_the_document() {
        let (d, [s, e1, e2, c1]) = doc();
        let cap = generated(e1, Role::EndCap);
        assert_eq!(
            TopoRef::Face(cap.clone()).describe(&d),
            "face Extrusion1 end cap"
        );
        assert_eq!(
            TopoRef::Edge(EdgeName::new(side(e2, s, 1), cap.clone(), vec![])).describe(&d),
            "edge between Extrusion1 end cap and Extrusion2 side (e1)"
        );
        let frag = FaceName::fragment(cap.clone(), c1, vec![side(c1, s, 2)]);
        assert_eq!(
            TopoRef::Face(frag).describe(&d),
            "face Extrusion1 end cap, piece split by Cut1 next to Cut1 side (e2)"
        );
        assert_eq!(
            TopoRef::Face(generated(FeatureId(42), Role::StartCap)).describe(&d),
            "face feature #42 start cap"
        );
        assert_eq!(
            TopoRef::Edge(EdgeName::new(
                cap,
                generated(e1, Role::StartCap),
                vec![side(FeatureId(42), FeatureId(41), 3)]
            ))
            .describe(&d),
            "edge between Extrusion1 start cap and Extrusion1 end cap, ending at feature #42 side (e3)"
        );
    }

    #[test]
    fn resolve_errors_display() {
        let (_, [_, e1, ..]) = doc();
        let face = TopoRef::Face(generated(e1, Role::EndCap));
        let missing = ResolveError::Missing {
            reference: face.clone(),
            reason: "Cut1 no longer splits this face".into(),
        };
        assert_eq!(
            missing.to_string(),
            "reference is missing: Cut1 no longer splits this face"
        );
        let ambiguous = ResolveError::Ambiguous {
            reference: face,
            matches: 2,
        };
        assert_eq!(ambiguous.to_string(), "reference matches 2 faces; pick one");
        let edge = TopoRef::Edge(EdgeName::new(
            generated(e1, Role::EndCap),
            generated(e1, Role::StartCap),
            vec![],
        ));
        let err: Box<dyn std::error::Error> = Box::new(ResolveError::Ambiguous {
            reference: edge,
            matches: 3,
        });
        assert_eq!(err.to_string(), "reference matches 3 edges; pick one");
    }
}
