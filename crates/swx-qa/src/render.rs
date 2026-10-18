//! Headless CPU renderer: orthographic isometric view, flat shading, feature edges.

use std::collections::HashMap;
use std::f64::consts::FRAC_1_SQRT_2;

use swx_kernel::Mesh;

/// An RGB color.
pub type Rgb = [u8; 3];

/// Background color.
pub const BACKGROUND: Rgb = [245, 245, 245];
/// Feature edge color.
pub const EDGE: Rgb = [20, 20, 20];
/// Surface color at full light.
const BASE: [f64; 3] = [150.0, 170.0, 200.0];
const AMBIENT: f64 = 0.3;
const DIFFUSE: f64 = 0.7;
/// Fixed directional light (towards the light, world space), unnormalized.
/// Chosen so the +X, +Y and +Z faces of a box all get different shades.
const LIGHT: [f64; 3] = [0.3, 0.5, 1.0];
/// Margin around the fitted model, as a fraction of each image dimension.
const MARGIN: f64 = 0.05;
/// Largest accepted image side, to keep allocations bounded.
pub const MAX_SIDE: u32 = 8192;

/// Render settings.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Options {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Crease angle in degrees: edges whose adjacent face normals differ by
    /// more than this are drawn.
    pub crease_deg: f64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            crease_deg: 30.0,
        }
    }
}

/// A rendered RGB image, row-major from the top-left pixel.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// `width * height` pixels.
    pub pixels: Vec<Rgb>,
}

impl Image {
    /// The pixel at column `x`, row `y`.
    pub fn get(&self, x: u32, y: u32) -> Rgb {
        self.pixels[(y * self.width + x) as usize]
    }

    /// Encodes the image as an 8-bit RGB PNG with no ancillary chunks, so the
    /// same image always gives the same bytes.
    pub fn to_png(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let mut enc = png::Encoder::new(&mut out, self.width, self.height);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        let data: Vec<u8> = self.pixels.iter().flatten().copied().collect();
        writer.write_image_data(&data).map_err(|e| e.to_string())?;
        writer.finish().map_err(|e| e.to_string())?;
        Ok(out)
    }
}

/// The isometric camera fitted to a mesh: maps model space to pixels.
///
/// Looks from +X+Y+Z towards the origin with +Z up on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    scale: f64,
    center: [f64; 2],
    width: u32,
    height: u32,
}

/// Unit vector from the scene towards the camera.
pub const TOWARD_CAMERA: [f64; 3] = [
    0.577_350_269_189_625_8,
    0.577_350_269_189_625_8,
    0.577_350_269_189_625_8,
];
/// Screen right, `normalize(Z × TOWARD_CAMERA)`.
const RIGHT: [f64; 3] = [-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0];
/// Screen up, `TOWARD_CAMERA × RIGHT`.
const UP: [f64; 3] = [
    -0.408_248_290_463_863,
    -0.408_248_290_463_863,
    0.816_496_580_927_726,
];

impl View {
    /// Fits the projected bounding box of `points` into the image with a 5 % margin.
    fn fit(points: &[[f64; 3]], width: u32, height: u32) -> Result<Self, String> {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in points {
            let s = [dot(*p, RIGHT), dot(*p, UP)];
            for k in 0..2 {
                lo[k] = lo[k].min(s[k]);
                hi[k] = hi[k].max(s[k]);
            }
        }
        let (w, h) = (hi[0] - lo[0], hi[1] - lo[1]);
        let avail = [
            f64::from(width) * (1.0 - 2.0 * MARGIN),
            f64::from(height) * (1.0 - 2.0 * MARGIN),
        ];
        let scale = match (w > 0.0, h > 0.0) {
            (true, true) => (avail[0] / w).min(avail[1] / h),
            (true, false) => avail[0] / w,
            (false, true) => avail[1] / h,
            (false, false) => return Err("mesh has no visible extent".into()),
        };
        Ok(Self {
            scale,
            center: [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0],
            width,
            height,
        })
    }

    /// Projects a model-space point to `[x, y, depth]`: continuous pixel
    /// coordinates (pixel `(i, j)` covers `[i, i+1) × [j, j+1)`) and depth
    /// towards the camera in model units (larger is nearer).
    pub fn project(&self, p: [f64; 3]) -> [f64; 3] {
        [
            f64::from(self.width) / 2.0 + (dot(p, RIGHT) - self.center[0]) * self.scale,
            f64::from(self.height) / 2.0 - (dot(p, UP) - self.center[1]) * self.scale,
            dot(p, TOWARD_CAMERA),
        ]
    }

    /// The pixel containing the projection of `p`, if it lies in the image.
    pub fn pixel(&self, p: [f64; 3]) -> Option<(u32, u32)> {
        let [x, y, _] = self.project(p);
        let (x, y) = (x.floor(), y.floor());
        let inside =
            x >= 0.0 && y >= 0.0 && x < f64::from(self.width) && y < f64::from(self.height);
        inside.then_some((x as u32, y as u32))
    }
}

/// Flat Lambert shade for a unit face normal: one directional light plus ambient.
pub fn shade(normal: [f64; 3]) -> Rgb {
    let l = normalize(LIGHT);
    let k = AMBIENT + DIFFUSE * dot(normal, l).max(0.0);
    BASE.map(|c| (c * k).round().clamp(0.0, 255.0) as u8)
}

/// Renders `mesh` and returns the image with the view used to draw it.
///
/// Errors on a mesh that fails [`Mesh::validate`], is empty, or has
/// non-finite coordinates, and on a zero or oversized image.
pub fn render(mesh: &Mesh, opts: &Options) -> Result<(Image, View), String> {
    let (w, h) = (opts.width, opts.height);
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!(
            "image size {w}x{h} must be between 1x1 and {MAX_SIDE}x{MAX_SIDE}"
        ));
    }
    if !(opts.crease_deg.is_finite() && (0.0..=180.0).contains(&opts.crease_deg)) {
        return Err(format!(
            "crease angle {} must be between 0 and 180 degrees",
            opts.crease_deg
        ));
    }
    mesh.validate().map_err(|e| format!("invalid mesh: {e}"))?;
    if mesh.triangle_count() == 0 {
        return Err("mesh is empty".into());
    }
    let pts: Vec<[f64; 3]> = mesh.positions.iter().map(|p| p.map(f64::from)).collect();
    if pts.iter().flatten().any(|c| !c.is_finite()) {
        return Err("mesh has non-finite coordinates".into());
    }
    let view = View::fit(&pts, w, h)?;
    let diag = bbox_diagonal(&pts);

    let tris: Vec<[usize; 3]> = mesh
        .indices
        .chunks_exact(3)
        .map(|t| [t[0] as usize, t[1] as usize, t[2] as usize])
        .collect();
    let normals: Vec<Option<[f64; 3]>> = tris.iter().map(|t| face_normal(&pts, *t)).collect();

    let mut img = Image {
        width: w,
        height: h,
        pixels: vec![BACKGROUND; (w * h) as usize],
    };
    let mut zbuf = vec![f64::NEG_INFINITY; (w * h) as usize];
    for (t, n) in tris.iter().zip(&normals) {
        let Some(n) = *n else { continue };
        if dot(n, TOWARD_CAMERA) <= 0.0 {
            continue; // back face
        }
        let v = t.map(|i| view.project(pts[i]));
        fill_triangle(&mut img, &mut zbuf, v, shade(n));
    }

    let cos_crease = opts.crease_deg.to_radians().cos();
    // A few pixels' worth of depth, so an edge is not hidden by its own faces.
    let bias = (0.005 * diag).max(3.0 / view.scale);
    for [a, b] in feature_edges(&pts, &tris, &normals, diag, cos_crease) {
        draw_line(&mut img, &zbuf, view.project(a), view.project(b), bias);
    }
    Ok((img, view))
}

/// Rasterizes one triangle with edge functions and a z-buffer.
///
/// Edge-function rasterization after Pineda, "A Parallel Algorithm for
/// Polygon Rasterization", SIGGRAPH 1988: a pixel center is inside when it
/// is on the inner side of all three edges. Depth is interpolated with the
/// same barycentric weights, which is exact for an orthographic view.
fn fill_triangle(img: &mut Image, zbuf: &mut [f64], v: [[f64; 3]; 3], color: Rgb) {
    let edge = |a: [f64; 3], b: [f64; 3], x: f64, y: f64| {
        (b[0] - a[0]) * (y - a[1]) - (b[1] - a[1]) * (x - a[0])
    };
    let area = edge(v[0], v[1], v[2][0], v[2][1]);
    if area == 0.0 {
        return;
    }
    let (w, h) = (f64::from(img.width), f64::from(img.height));
    let x0 = v
        .iter()
        .map(|p| p[0])
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0);
    let x1 = v
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .min(w);
    let y0 = v
        .iter()
        .map(|p| p[1])
        .fold(f64::INFINITY, f64::min)
        .floor()
        .max(0.0);
    let y1 = v
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max)
        .ceil()
        .min(h);
    let (x0, x1, y0, y1) = (x0 as u32, x1 as u32, y0 as u32, y1 as u32);
    for py in y0..y1 {
        for px in x0..x1 {
            let (x, y) = (f64::from(px) + 0.5, f64::from(py) + 0.5);
            let l0 = edge(v[1], v[2], x, y) / area;
            let l1 = edge(v[2], v[0], x, y) / area;
            let l2 = edge(v[0], v[1], x, y) / area;
            if l0 < 0.0 || l1 < 0.0 || l2 < 0.0 {
                continue;
            }
            let z = l0 * v[0][2] + l1 * v[1][2] + l2 * v[2][2];
            let i = (py * img.width + px) as usize;
            if z > zbuf[i] {
                zbuf[i] = z;
                img.pixels[i] = color;
            }
        }
    }
}

/// Draws a line in [`EDGE`] color where it is not behind the z-buffer by more
/// than `bias`. Samples an even number of steps so the exact midpoint is hit.
fn draw_line(img: &mut Image, zbuf: &[f64], a: [f64; 3], b: [f64; 3], bias: f64) {
    let len = (b[0] - a[0]).abs().max((b[1] - a[1]).abs());
    let steps = 2 * (len.ceil() as u32).max(1);
    for s in 0..=steps {
        let t = f64::from(s) / f64::from(steps);
        let p = [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t);
        let (x, y) = (p[0].floor(), p[1].floor());
        if x < 0.0 || y < 0.0 || x >= f64::from(img.width) || y >= f64::from(img.height) {
            continue;
        }
        let i = (y as u32 * img.width + x as u32) as usize;
        if p[2] + bias >= zbuf[i] {
            img.pixels[i] = EDGE;
        }
    }
}

/// Edges to draw, as model-space segments in a deterministic order.
///
/// Vertices are first welded by position (tolerance `1e-6 × diag`) so that
/// seams of duplicated vertices on smooth faces are not mistaken for
/// boundaries. Then, following the feature-edge classification in
/// Akenine-Möller et al., *Real-Time Rendering*, 4th ed., ch. 15, an edge is
/// drawn when it is a boundary (one face) or non-manifold (more than two),
/// a crease (adjacent face normals differ by more than the crease angle), or
/// a silhouette (one adjacent face front-facing, the other back-facing).
fn feature_edges(
    pts: &[[f64; 3]],
    tris: &[[usize; 3]],
    normals: &[Option<[f64; 3]>],
    diag: f64,
    cos_crease: f64,
) -> Vec<[[f64; 3]; 2]> {
    let welded = weld(pts, 1e-6 * diag);
    let mut faces_of: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (f, (t, n)) in tris.iter().zip(normals).enumerate() {
        if n.is_none() {
            continue;
        }
        let w = t.map(|i| welded[i]);
        for k in 0..3 {
            let (a, b) = (w[k], w[(k + 1) % 3]);
            if a != b {
                faces_of.entry((a.min(b), a.max(b))).or_default().push(f);
            }
        }
    }
    let mut keys: Vec<_> = faces_of
        .iter()
        .filter(|(_, fs)| match fs.as_slice() {
            [f, g] => {
                let (nf, ng) = (normals[*f].unwrap(), normals[*g].unwrap());
                let front = |n| dot(n, TOWARD_CAMERA) > 0.0;
                dot(nf, ng) < cos_crease || front(nf) != front(ng)
            }
            _ => true,
        })
        .map(|(k, _)| *k)
        .collect();
    keys.sort_unstable();
    keys.into_iter().map(|(a, b)| [pts[a], pts[b]]).collect()
}

/// Maps each vertex to the index of the first vertex within `tol` of it,
/// using a uniform grid of cell size `tol` and checking neighboring cells.
fn weld(pts: &[[f64; 3]], tol: f64) -> Vec<usize> {
    let cell = |p: [f64; 3]| p.map(|c| (c / tol).floor() as i64);
    let mut grid: HashMap<[i64; 3], Vec<usize>> = HashMap::new();
    let mut out = Vec::with_capacity(pts.len());
    for (i, &p) in pts.iter().enumerate() {
        let c = cell(p);
        let mut found = None;
        'search: for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if let Some(list) = grid.get(&[c[0] + dx, c[1] + dy, c[2] + dz]) {
                        if let Some(&j) = list.iter().find(|&&j| dist(pts[j], p) <= tol) {
                            found = Some(j);
                            break 'search;
                        }
                    }
                }
            }
        }
        match found {
            Some(j) => out.push(j),
            None => {
                grid.entry(c).or_default().push(i);
                out.push(i);
            }
        }
    }
    out
}

fn face_normal(pts: &[[f64; 3]], t: [usize; 3]) -> Option<[f64; 3]> {
    let (a, b, c) = (pts[t[0]], pts[t[1]], pts[t[2]]);
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    (dot(n, n) > 0.0).then(|| normalize(n))
}

fn bbox_diagonal(pts: &[[f64; 3]]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for p in pts {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    dist(lo, hi)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    dot(d, d).sqrt()
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = dot(v, v).sqrt();
    v.map(|c| c / l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;
    use std::collections::BTreeSet;

    fn small() -> Options {
        Options {
            width: 200,
            height: 200,
            ..Options::default()
        }
    }

    fn face_shades(img: &Image) -> BTreeSet<Rgb> {
        img.pixels
            .iter()
            .copied()
            .filter(|&c| c != BACKGROUND && c != EDGE)
            .collect()
    }

    fn px(img: &Image, view: &View, p: [f64; 3]) -> Rgb {
        let (x, y) = view.pixel(p).expect("point projects inside the image");
        img.get(x, y)
    }

    fn mid(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
        [0, 1, 2].map(|k| (a[k] + b[k]) / 2.0)
    }

    #[test]
    fn cube_corners_shades_and_centering() {
        let (img, _) = render(&fixtures::cube(), &small()).unwrap();
        for (x, y) in [(0, 0), (199, 0), (0, 199), (199, 199)] {
            assert_eq!(img.get(x, y), BACKGROUND, "corner ({x}, {y})");
        }
        assert_eq!(face_shades(&img).len(), 3, "{:?}", face_shades(&img));

        let (mut lo, mut hi) = ([u32::MAX; 2], [0u32; 2]);
        for y in 0..img.height {
            for x in 0..img.width {
                if img.get(x, y) != BACKGROUND {
                    lo = [lo[0].min(x), lo[1].min(y)];
                    hi = [hi[0].max(x), hi[1].max(y)];
                }
            }
        }
        for k in 0..2 {
            let center = f64::from(lo[k] + hi[k] + 1) / 2.0;
            assert!((center - 100.0).abs() <= 2.0, "axis {k}: {lo:?}..{hi:?}");
        }
    }

    #[test]
    fn nearer_triangle_wins_in_both_draw_orders() {
        // A: horizontal through the origin (normal +Z). B: facing the camera,
        // 3 mm behind the origin along the view direction.
        let a = [[-5.0, -5.0, 0.0], [5.0, -5.0, 0.0], [0.0, 5.0, 0.0]];
        let at =
            |r: f64, u: f64| [0, 1, 2].map(|k| r * RIGHT[k] + u * UP[k] - 3.0 * TOWARD_CAMERA[k]);
        let b = [at(-6.0, -6.0), at(6.0, -6.0), at(0.0, 6.0)];
        let mesh = |first: [[f64; 3]; 3], second: [[f64; 3]; 3]| {
            let positions: Vec<[f32; 3]> = first
                .iter()
                .chain(&second)
                .map(|p| p.map(|c| c as f32))
                .collect();
            Mesh {
                normals: vec![[0.0, 0.0, 1.0]; 6],
                positions,
                indices: (0..6).collect(),
            }
        };
        let near = shade([0.0, 0.0, 1.0]);
        assert_ne!(near, shade(TOWARD_CAMERA));
        for m in [mesh(a, b), mesh(b, a)] {
            let (img, view) = render(&m, &small()).unwrap();
            assert_eq!(px(&img, &view, [0.0, 0.0, 0.0]), near);
        }
    }

    #[test]
    fn cube_edges_drawn_but_not_face_diagonals() {
        let m = fixtures::cube();
        let (img, view) = render(&m, &small()).unwrap();
        // Edge between the +X and +Z faces.
        assert_eq!(px(&img, &view, [10.0, 5.0, 10.0]), EDGE);

        // The diagonal shared by the two triangles of the +Z face.
        let top: Vec<Vec<[f64; 3]>> = m
            .indices
            .chunks_exact(3)
            .filter(|t| t.iter().all(|&i| m.positions[i as usize][2] == 10.0))
            .map(|t| {
                t.iter()
                    .map(|&i| m.positions[i as usize].map(f64::from))
                    .collect()
            })
            .collect();
        assert_eq!(top.len(), 2);
        let shared: Vec<[f64; 3]> = top[0]
            .iter()
            .filter(|p| top[1].contains(p))
            .copied()
            .collect();
        assert_eq!(shared.len(), 2);
        let c = px(&img, &view, mid(shared[0], shared[1]));
        assert_ne!(c, EDGE);
        assert_ne!(c, BACKGROUND);
    }

    #[test]
    fn cylinder_seam_hidden_rim_drawn() {
        let (img, view) = render(&fixtures::cylinder(), &small()).unwrap();
        let ring = |a: f64, z: f64| [5.0 * a.cos(), 5.0 * a.sin(), z];
        let seam = fixtures::CYLINDER_SEAM_ANGLE;
        let c = px(&img, &view, ring(seam, 5.0));
        assert_ne!(c, EDGE);
        assert_ne!(c, BACKGROUND);
        // The rim segment starting at the seam is one of the two nearest the camera.
        let step = std::f64::consts::TAU / fixtures::CYLINDER_SEGMENTS as f64;
        let rim = mid(ring(seam, 10.0), ring(seam + step, 10.0));
        assert_eq!(px(&img, &view, rim), EDGE);
    }

    #[test]
    fn renders_are_byte_identical_without_ancillary_chunks() {
        let png1 = render(&fixtures::l_bracket(), &small())
            .unwrap()
            .0
            .to_png()
            .unwrap();
        let png2 = render(&fixtures::l_bracket(), &small())
            .unwrap()
            .0
            .to_png()
            .unwrap();
        assert_eq!(png1, png2);
        // Walk the chunks after the 8-byte signature: only IHDR, IDAT, IEND.
        let mut i = 8;
        while i < png1.len() {
            let len = u32::from_be_bytes(png1[i..i + 4].try_into().unwrap()) as usize;
            let kind = &png1[i + 4..i + 8];
            assert!([&b"IHDR"[..], b"IDAT", b"IEND"].contains(&kind), "{kind:?}");
            i += 12 + len;
        }
    }

    #[test]
    fn default_size_renders_are_under_500_kb() {
        for name in fixtures::FIXTURE_NAMES {
            let (img, _) = render(&fixtures::fixture(name).unwrap(), &Options::default()).unwrap();
            assert_eq!((img.width, img.height), (800, 600));
            let n = img.to_png().unwrap().len();
            assert!(n < 500 * 1024, "{name}: {n} bytes");
        }
    }

    #[test]
    fn bad_meshes_and_options_are_errors() {
        assert!(render(&Mesh::default(), &small())
            .unwrap_err()
            .contains("empty"));
        let mut broken = fixtures::cube();
        broken.indices.push(0);
        assert!(render(&broken, &small())
            .unwrap_err()
            .contains("invalid mesh"));
        let mut nan = fixtures::cube();
        nan.positions[0][0] = f32::NAN;
        assert!(render(&nan, &small()).is_err());
        let zero = Options {
            width: 0,
            ..small()
        };
        assert!(render(&fixtures::cube(), &zero).is_err());
        let crease = Options {
            crease_deg: -1.0,
            ..small()
        };
        assert!(render(&fixtures::cube(), &crease).is_err());
    }
}
