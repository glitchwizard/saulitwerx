//! Plots a solved [`Sketch`] to an [`Image`]: geometry, point dots and
//! dimension values, top-down with x right and y up.
//!
//! Builders call [`plot`] from a scratch program on any sketch, or use the
//! built-in [`fixture`]s through `swx-qa sketch`.

use std::f64::consts::TAU;

use swx_sketch::solver::SolveOptions;
use swx_sketch::{Constraint, GeomId, Geometry, Line, Point, PointRef, Sketch, SketchError};

use crate::render::{Image, Rgb, BACKGROUND, MAX_SIDE};

/// Geometry (line, circle, arc) color.
pub const GEOMETRY: Rgb = [30, 60, 110];
/// Point and line-endpoint dot color.
pub const DOT: Rgb = [200, 40, 40];
/// Dimension label color.
pub const LABEL: Rgb = [20, 120, 40];
/// Margin around the fitted sketch, as a fraction of each image dimension
/// per side. Larger than the minimum 5 % so labels outside the geometry fit.
const MARGIN: f64 = 0.1;
/// Dot radius in pixels: dots are `2 * DOT_RADIUS + 1` = 5 px across.
const DOT_RADIUS: i64 = 2;
/// Each font cell is drawn as a `TEXT_SCALE` × `TEXT_SCALE` pixel block.
const TEXT_SCALE: i64 = 2;
/// Gap in pixels between a label and the geometry it labels.
const LABEL_GAP: f64 = 4.0;
/// Smallest margins in pixels `(x, y)`, so a label of up to 3 characters
/// beside the outermost geometry fits in small images (each capped at 25 %
/// of its dimension): 3 glyph cells wide, or one line tall, plus two gaps.
const MIN_MARGIN_PX: [f64; 2] = [
    ((6 * 3 - 1) * TEXT_SCALE) as f64 + 2.0 * LABEL_GAP,
    (7 * TEXT_SCALE) as f64 + 2.0 * LABEL_GAP,
];

/// Plot settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotOptions {
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

impl Default for PlotOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
        }
    }
}

/// The uniform model-to-pixel transform fitted to a sketch.
///
/// Pixel coordinates are continuous, with `(0, 0)` at the top-left corner of
/// the image; pixel `(i, j)` covers `[i, i+1) × [j, j+1)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    scale: f64,
    center: [f64; 2],
    width: u32,
    height: u32,
}

impl Fit {
    /// Fits the model-space box `min..max` into `width × height` with a
    /// [`MARGIN`] on each side, but at least [`MIN_MARGIN_PX`] (capped at
    /// 25 %). A box with zero extent along an axis is fitted along the other
    /// one; a single point gets scale 1.
    pub fn new(min: [f64; 2], max: [f64; 2], width: u32, height: u32) -> Self {
        let usable = [0, 1].map(|i| {
            let side = f64::from([width, height][i]);
            let margin = (side * MARGIN).max(MIN_MARGIN_PX[i].min(side * 0.25));
            side - 2.0 * margin
        });
        let scale = (0..2)
            .filter(|&i| max[i] - min[i] > 0.0)
            .map(|i| usable[i] / (max[i] - min[i]))
            .fold(f64::INFINITY, f64::min);
        Self {
            scale: if scale.is_finite() { scale } else { 1.0 },
            center: [(min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0],
            width,
            height,
        }
    }

    /// Model `(x, y)` to continuous pixel coordinates (y flipped to point up).
    pub fn pixel(&self, x: f64, y: f64) -> [f64; 2] {
        [
            f64::from(self.width) / 2.0 + (x - self.center[0]) * self.scale,
            f64::from(self.height) / 2.0 - (y - self.center[1]) * self.scale,
        ]
    }

    /// Pixels per model unit.
    pub fn scale(&self) -> f64 {
        self.scale
    }
}

/// Formats a dimension value with up to 3 decimals, trailing zeros trimmed.
pub fn format_value(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}

/// The label for a dimensional constraint (`40`, `R5`, `Ø10`), or `None` for
/// a non-dimensional one.
pub fn label_text(c: &Constraint) -> Option<String> {
    match *c {
        Constraint::Distance { value, .. } => Some(format_value(value)),
        Constraint::Radius { value, .. } => Some(format!("R{}", format_value(value))),
        Constraint::Diameter { value, .. } => Some(format!("Ø{}", format_value(value))),
        _ => None,
    }
}

/// Characters with a glyph in the built-in font.
pub const GLYPH_CHARS: &str = "0123456789.-°RØ";

/// The 5 × 7 bitmap of `c`, one row per byte from the top, bit 4 leftmost;
/// `None` if the font has no glyph for it. Drawn for this crate.
pub fn glyph(c: char) -> Option<[u8; 7]> {
    Some(match c {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '°' => [0x0C, 0x12, 0x12, 0x0C, 0x00, 0x00, 0x00],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'Ø' => [0x0D, 0x12, 0x15, 0x15, 0x15, 0x09, 0x16],
        _ => return None,
    })
}

/// Model coordinates of a point reference, or `None` if it doesn't resolve.
fn point_xy(s: &Sketch, p: PointRef) -> Option<Point> {
    let id = match p {
        PointRef::Point(id) | PointRef::Start(id) | PointRef::End(id) | PointRef::Center(id) => id,
    };
    match (s.get(id).ok()?, p) {
        (Geometry::Point(pt), PointRef::Point(_)) => Some(pt),
        (Geometry::Line(l), PointRef::Start(_)) => Some(l.start),
        (Geometry::Line(l), PointRef::End(_)) => Some(l.end),
        (Geometry::Circle(c), PointRef::Center(_)) => Some(c.center),
        (Geometry::Arc(a), PointRef::Center(_)) => Some(a.center),
        _ => None,
    }
}

/// All entities, in id order.
fn entities(s: &Sketch) -> Result<Vec<Geometry>, String> {
    (0..s.len() as u32)
        .map(|i| s.get(GeomId(i)).map_err(|e| e.to_string()))
        .collect()
}

/// Polyline through a circular arc from angle `a0` sweeping `sweep` radians
/// counter-clockwise, with enough segments for about 2 px per segment.
fn arc_points(c: Point, r: f64, a0: f64, sweep: f64, scale: f64) -> Vec<Point> {
    let n = ((r * scale * sweep / 2.0).ceil() as usize).clamp(8, 4096);
    (0..=n)
        .map(|k| {
            let a = a0 + sweep * k as f64 / n as f64;
            Point {
                x: c.x + r * a.cos(),
                y: c.y + r * a.sin(),
            }
        })
        .collect()
}

/// Model-space bounding box of all geometry, or `None` for an empty sketch.
///
/// An arc's box holds its end points plus each axis extreme (angle `kπ/2`)
/// inside its counter-clockwise sweep.
fn bounds(geoms: &[Geometry]) -> Option<([f64; 2], [f64; 2])> {
    let at = |c: Point, r: f64, a: f64| Point {
        x: c.x + r * a.cos(),
        y: c.y + r * a.sin(),
    };
    let mut pts = Vec::new();
    for g in geoms {
        match *g {
            Geometry::Point(p) => pts.push(p),
            Geometry::Line(l) => pts.extend([l.start, l.end]),
            Geometry::Circle(c) => {
                pts.extend([0.0, 1.0, 2.0, 3.0].map(|k| at(c.center, c.radius, k * TAU / 4.0)))
            }
            Geometry::Arc(a) => {
                pts.extend([a.start_point(), a.end_point()]);
                let quarter = TAU / 4.0;
                let mut k = (a.start_angle / quarter).ceil();
                while k * quarter <= a.end_angle {
                    pts.push(at(a.center, a.radius, k * quarter));
                    k += 1.0;
                }
            }
        }
    }
    let first = pts.first()?;
    let init = ([first.x, first.y], [first.x, first.y]);
    Some(pts.iter().fold(init, |(lo, hi), p| {
        (
            [lo[0].min(p.x), lo[1].min(p.y)],
            [hi[0].max(p.x), hi[1].max(p.y)],
        )
    }))
}

/// A drawing surface over an [`Image`] that ignores out-of-bounds pixels.
struct Canvas(Image);

impl Canvas {
    fn set(&mut self, x: i64, y: i64, c: Rgb) {
        let (w, h) = (i64::from(self.0.width), i64::from(self.0.height));
        if (0..w).contains(&x) && (0..h).contains(&y) {
            self.0.pixels[(y * w + x) as usize] = c;
        }
    }

    /// Draws the segment between two continuous pixel positions with
    /// Bresenham's line algorithm (J. E. Bresenham, "Algorithm for computer
    /// control of a digital plotter", IBM Systems Journal 4(1), 1965), in
    /// its all-octant integer error form.
    fn line(&mut self, a: [f64; 2], b: [f64; 2], c: Rgb) {
        let (mut x0, mut y0) = (a[0].floor() as i64, a[1].floor() as i64);
        let (x1, y1) = (b[0].floor() as i64, b[1].floor() as i64);
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = ((x1 - x0).signum(), (y1 - y0).signum());
        let mut err = dx + dy;
        loop {
            self.set(x0, y0, c);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    /// A filled disk of radius [`DOT_RADIUS`] centered on the pixel at `p`.
    fn dot(&mut self, p: [f64; 2], c: Rgb) {
        let (x, y) = (p[0].floor() as i64, p[1].floor() as i64);
        for dy in -DOT_RADIUS..=DOT_RADIUS {
            for dx in -DOT_RADIUS..=DOT_RADIUS {
                if dx * dx + dy * dy <= DOT_RADIUS * DOT_RADIUS + 1 {
                    self.set(x + dx, y + dy, c);
                }
            }
        }
    }

    /// Draws `text` with its top-left corner at pixel `(x, y)`. Characters
    /// without a glyph are skipped (but keep their space).
    fn text(&mut self, text: &str, x: i64, y: i64, c: Rgb) {
        for (i, ch) in text.chars().enumerate() {
            let Some(rows) = glyph(ch) else { continue };
            let gx = x + i as i64 * 6 * TEXT_SCALE;
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..5 {
                    if bits & (0x10 >> col) != 0 {
                        for sy in 0..TEXT_SCALE {
                            for sx in 0..TEXT_SCALE {
                                self.set(
                                    gx + col * TEXT_SCALE + sx,
                                    y + row as i64 * TEXT_SCALE + sy,
                                    c,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Pixel size `(width, height)` of `text` as [`Canvas::text`] draws it.
fn text_size(text: &str) -> (f64, f64) {
    let n = text.chars().count() as i64;
    let w = if n == 0 { 0 } else { (6 * n - 1) * TEXT_SCALE };
    (w as f64, (7 * TEXT_SCALE) as f64)
}

/// Draws `text` centered next to `anchor`, pushed out along the unit screen
/// direction `dir` so its box clears the anchor by [`LABEL_GAP`].
fn place_label(cv: &mut Canvas, text: &str, anchor: [f64; 2], dir: [f64; 2]) {
    let (w, h) = text_size(text);
    let off = dir[0].abs() * w / 2.0 + dir[1].abs() * h / 2.0 + LABEL_GAP;
    let cx = anchor[0] + dir[0] * off;
    let cy = anchor[1] + dir[1] * off;
    cv.text(
        text,
        (cx - w / 2.0).round() as i64,
        (cy - h / 2.0).round() as i64,
        LABEL,
    );
}

/// Unit vector along `v`, or `fallback` if `v` is (nearly) zero.
fn unit(v: [f64; 2], fallback: [f64; 2]) -> [f64; 2] {
    let n = v[0].hypot(v[1]);
    if n > 1e-9 {
        [v[0] / n, v[1] / n]
    } else {
        fallback
    }
}

/// Plots `sketch` top-down (x right, y up), uniformly scaled to fit the
/// image with a 10 % margin per side, widened to [`MIN_MARGIN_PX`] in small
/// images.
///
/// Draws lines, circles and arcs (counter-clockwise from start to end
/// angle), dots on point entities and line endpoints, and the value of every
/// `Distance`, `Radius` and `Diameter` constraint: a distance label sits next
/// to the midpoint of its two points, on the side away from the sketch
/// center; a radius or diameter label sits outside the circle at 45°, or
/// outside the arc at its mid-angle.
///
/// An empty sketch gives a background-only image. Errors only for an image
/// size outside `1x1..=MAX_SIDE x MAX_SIDE`.
pub fn plot(sketch: &Sketch, opts: &PlotOptions) -> Result<Image, String> {
    let (w, h) = (opts.width, opts.height);
    if w == 0 || h == 0 || w > MAX_SIDE || h > MAX_SIDE {
        return Err(format!(
            "image size {w}x{h} must be between 1x1 and {MAX_SIDE}x{MAX_SIDE}"
        ));
    }
    let mut cv = Canvas(Image {
        width: w,
        height: h,
        pixels: vec![BACKGROUND; w as usize * h as usize],
    });
    let geoms = entities(sketch)?;
    let Some((min, max)) = bounds(&geoms) else {
        return Ok(cv.0);
    };
    let fit = Fit::new(min, max, w, h);
    let px = |p: Point| fit.pixel(p.x, p.y);
    let polyline = |cv: &mut Canvas, pts: &[Point]| {
        for s in pts.windows(2) {
            cv.line(px(s[0]), px(s[1]), GEOMETRY);
        }
    };

    for g in &geoms {
        match *g {
            Geometry::Point(_) => {}
            Geometry::Line(Line { start, end }) => polyline(&mut cv, &[start, end]),
            Geometry::Circle(c) => polyline(
                &mut cv,
                &arc_points(c.center, c.radius, 0.0, TAU, fit.scale()),
            ),
            Geometry::Arc(a) => polyline(
                &mut cv,
                &arc_points(a.center, a.radius, a.start_angle, a.sweep(), fit.scale()),
            ),
        }
    }
    for g in &geoms {
        match *g {
            Geometry::Point(p) => cv.dot(px(p), DOT),
            Geometry::Line(l) => {
                cv.dot(px(l.start), DOT);
                cv.dot(px(l.end), DOT);
            }
            _ => {}
        }
    }

    let center = fit.pixel((min[0] + max[0]) / 2.0, (min[1] + max[1]) / 2.0);
    for c in sketch.constraints() {
        let Some(text) = label_text(c) else { continue };
        let (anchor, dir) = match *c {
            Constraint::Distance { a, b, .. } => {
                let (Some(a), Some(b)) = (point_xy(sketch, a), point_xy(sketch, b)) else {
                    continue;
                };
                let (pa, pb) = (px(a), px(b));
                let mid = [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0];
                let mut n = unit([pa[1] - pb[1], pb[0] - pa[0]], [0.0, -1.0]);
                let out = [mid[0] - center[0], mid[1] - center[1]];
                if n[0] * out[0] + n[1] * out[1] < 0.0 {
                    n = [-n[0], -n[1]];
                }
                (mid, n)
            }
            Constraint::Radius { geom, .. } | Constraint::Diameter { geom, .. } => {
                let (c, r, a) = match sketch.get(geom) {
                    Ok(Geometry::Circle(c)) => (c.center, c.radius, TAU / 8.0),
                    Ok(Geometry::Arc(a)) => (a.center, a.radius, a.start_angle + a.sweep() / 2.0),
                    _ => continue,
                };
                let on = px(Point {
                    x: c.x + r * a.cos(),
                    y: c.y + r * a.sin(),
                });
                (on, [a.cos(), -a.sin()])
            }
            _ => continue,
        };
        place_label(&mut cv, &text, anchor, dir);
    }
    Ok(cv.0)
}

/// Names accepted by [`fixture`], in the order `--help` lists them.
pub const FIXTURE_NAMES: [&str; 2] = ["rectangle", "triangle-345"];

/// Returns the named built-in sketch, solved, or `None` for an unknown name.
pub fn fixture(name: &str) -> Option<Result<Sketch, String>> {
    match name {
        "rectangle" => Some(rectangle()),
        "triangle-345" => Some(triangle_345()),
        _ => None,
    }
}

fn line(s: &mut Sketch, x0: f64, y0: f64, x1: f64, y1: f64) -> Result<GeomId, SketchError> {
    let p = Point::new;
    s.add(Geometry::Line(Line::new(p(x0, y0)?, p(x1, y1)?)?))
}

/// Builds a closed loop of lines from approximate end points, joined end to
/// start by coincident constraints.
fn closed_loop<const N: usize>(
    s: &mut Sketch,
    ends: [(f64, f64, f64, f64); N],
) -> Result<[GeomId; N], SketchError> {
    let mut ids = [GeomId(0); N];
    for (id, (x0, y0, x1, y1)) in ids.iter_mut().zip(ends) {
        *id = line(s, x0, y0, x1, y1)?;
    }
    for i in 0..N {
        s.add_constraint(Constraint::Coincident(
            PointRef::End(ids[i]),
            PointRef::Start(ids[(i + 1) % N]),
        ))?;
    }
    Ok(ids)
}

/// Solves `s`, failing unless the solver converged.
fn solved(mut s: Sketch) -> Result<Sketch, String> {
    let report = s
        .solve(SolveOptions::default())
        .map_err(|e| e.to_string())?;
    if report.converged {
        Ok(s)
    } else {
        Err(format!("fixture did not converge: {report:?}"))
    }
}

/// The 40 × 25 rectangle of the M2-03 combined test, solved from the same
/// perturbed start: bottom-left corner fixed at the origin, horizontal and
/// vertical sides, bottom 40 and right 25.
pub fn rectangle() -> Result<Sketch, String> {
    let build = || -> Result<Sketch, SketchError> {
        let mut s = Sketch::new();
        let [bottom, right, top, left] = closed_loop(
            &mut s,
            [
                (2.0, -3.0, 44.0, 1.5),
                (43.0, -2.0, 37.0, 28.0),
                (38.0, 27.0, -4.0, 22.0),
                (-5.0, 24.0, 3.0, 4.0),
            ],
        )?;
        for c in [
            Constraint::Horizontal(bottom),
            Constraint::Horizontal(top),
            Constraint::Vertical(right),
            Constraint::Vertical(left),
            fixed_origin(PointRef::Start(bottom)),
            distance(bottom, 40.0),
            distance(right, 25.0),
        ] {
            s.add_constraint(c)?;
        }
        Ok(s)
    };
    solved(build().map_err(|e| e.to_string())?)
}

/// A 3-4-5 right triangle, solved from a perturbed start: right-angle corner
/// fixed at the origin, horizontal leg 4, vertical leg 3, hypotenuse free.
pub fn triangle_345() -> Result<Sketch, String> {
    let build = || -> Result<Sketch, SketchError> {
        let mut s = Sketch::new();
        let [base, side, _hyp] = closed_loop(
            &mut s,
            [
                (0.3, -0.4, 4.6, 0.2),
                (4.2, 0.5, 3.6, 3.4),
                (4.4, 2.7, -0.5, 0.3),
            ],
        )?;
        for c in [
            Constraint::Horizontal(base),
            Constraint::Vertical(side),
            fixed_origin(PointRef::Start(base)),
            distance(base, 4.0),
            distance(side, 3.0),
        ] {
            s.add_constraint(c)?;
        }
        Ok(s)
    };
    solved(build().map_err(|e| e.to_string())?)
}

fn fixed_origin(point: PointRef) -> Constraint {
    Constraint::Fixed {
        point,
        x: 0.0,
        y: 0.0,
    }
}

fn distance(line: GeomId, value: f64) -> Constraint {
    Constraint::Distance {
        a: PointRef::Start(line),
        b: PointRef::End(line),
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swx_sketch::{Arc, Circle};

    const W: u32 = 400;
    const H: u32 = 300;

    fn opts() -> PlotOptions {
        PlotOptions {
            width: W,
            height: H,
        }
    }

    fn lines(s: &Sketch) -> Vec<Line> {
        entities(s)
            .unwrap()
            .into_iter()
            .filter_map(|g| match g {
                Geometry::Line(l) => Some(l),
                _ => None,
            })
            .collect()
    }

    /// Non-background pixels within `r` px (Chebyshev) of `p`.
    fn drawn_near(img: &Image, p: [f64; 2], r: i64, color: Option<Rgb>) -> usize {
        let (x, y) = (p[0].floor() as i64, p[1].floor() as i64);
        let mut n = 0;
        for yy in y - r..=y + r {
            for xx in x - r..=x + r {
                if xx < 0 || yy < 0 || xx >= i64::from(img.width) || yy >= i64::from(img.height) {
                    continue;
                }
                let c = img.get(xx as u32, yy as u32);
                if c != BACKGROUND && color.map_or(true, |k| k == c) {
                    n += 1;
                }
            }
        }
        n
    }

    fn rect_fit(s: &Sketch) -> Fit {
        let (min, max) = bounds(&entities(s).unwrap()).unwrap();
        Fit::new(min, max, W, H)
    }

    #[test]
    fn rectangle_corners_and_sides_land_where_expected() {
        let s = rectangle().unwrap();
        let img = plot(&s, &opts()).unwrap();
        let fit = rect_fit(&s);
        let corners = [(0.0, 0.0), (40.0, 0.0), (40.0, 25.0), (0.0, 25.0)];
        for (x, y) in corners {
            let p = fit.pixel(x, y);
            assert!(
                drawn_near(&img, p, 2, Some(DOT)) > 0,
                "no dot within 2 px of corner ({x}, {y}) at {p:?}"
            );
            // The margin is at least 5 % per side.
            assert!(p[0] >= 0.05 * f64::from(W) && p[0] <= 0.95 * f64::from(W));
            assert!(p[1] >= 0.05 * f64::from(H) && p[1] <= 0.95 * f64::from(H));
        }
        for l in lines(&s) {
            let p = fit.pixel((l.start.x + l.end.x) / 2.0, (l.start.y + l.end.y) / 2.0);
            assert_eq!(
                img.get(p[0] as u32, p[1] as u32),
                GEOMETRY,
                "side midpoint {p:?} not drawn"
            );
        }
        // y up: the bottom side (y = 0) is lower on screen than the top.
        assert!(fit.pixel(0.0, 0.0)[1] > fit.pixel(0.0, 25.0)[1]);
    }

    #[test]
    fn distance_40_label_is_by_the_bottom_side() {
        let s = rectangle().unwrap();
        let img = plot(&s, &opts()).unwrap();
        let mid = rect_fit(&s).pixel(20.0, 0.0);
        assert!(drawn_near(&img, mid, 20, Some(LABEL)) > 0);
    }

    #[test]
    fn glyphs_cover_every_label_character() {
        for c in GLYPH_CHARS.chars() {
            assert!(glyph(c).is_some(), "{c}");
        }
        for v in [0.5, 25.0, 40.0, 123.456] {
            for c in [
                Constraint::Distance {
                    a: PointRef::Point(GeomId(0)),
                    b: PointRef::Point(GeomId(1)),
                    value: v,
                },
                Constraint::Radius {
                    geom: GeomId(0),
                    value: v,
                },
                Constraint::Diameter {
                    geom: GeomId(0),
                    value: v,
                },
            ] {
                let t = label_text(&c).unwrap();
                for ch in t.chars() {
                    assert!(glyph(ch).is_some(), "no glyph for {ch:?} in {t:?}");
                }
            }
        }
    }

    #[test]
    fn values_are_formatted_with_up_to_3_decimals() {
        let cases = [
            (0.5, "0.5"),
            (25.0, "25"),
            (40.0, "40"),
            (123.456, "123.456"),
            (1.23456, "1.235"),
            (2.0004, "2"),
            (-0.0001, "0"),
        ];
        for (v, s) in cases {
            assert_eq!(format_value(v), s);
        }
        let r = Constraint::Radius {
            geom: GeomId(0),
            value: 5.0,
        };
        let d = Constraint::Diameter {
            geom: GeomId(0),
            value: 2.5,
        };
        assert_eq!(label_text(&r).unwrap(), "R5");
        assert_eq!(label_text(&d).unwrap(), "Ø2.5");
        assert_eq!(label_text(&Constraint::Horizontal(GeomId(0))), None);
    }

    #[test]
    fn plotting_is_deterministic() {
        for name in FIXTURE_NAMES {
            let a = plot(&fixture(name).unwrap().unwrap(), &opts()).unwrap();
            let b = plot(&fixture(name).unwrap().unwrap(), &opts()).unwrap();
            assert_eq!(a.to_png().unwrap(), b.to_png().unwrap(), "{name}");
        }
    }

    #[test]
    fn triangle_fixture_solves_to_3_4_5() {
        let s = triangle_345().unwrap();
        let ls = lines(&s);
        let lens: Vec<f64> = ls.iter().map(Line::length).collect();
        for (got, want) in lens.iter().zip([4.0, 3.0, 5.0]) {
            assert!((got - want).abs() < 1e-9, "{lens:?}");
        }
    }

    #[test]
    fn labels_are_not_clipped_in_small_images() {
        for name in FIXTURE_NAMES {
            let img = plot(
                &fixture(name).unwrap().unwrap(),
                &PlotOptions {
                    width: 120,
                    height: 90,
                },
            )
            .unwrap();
            let (w, h) = (img.width, img.height);
            let edge = (0..w)
                .flat_map(|x| [(x, 0), (x, h - 1)])
                .chain((0..h).flat_map(|y| [(0, y), (w - 1, y)]));
            for (x, y) in edge {
                assert_eq!(img.get(x, y), BACKGROUND, "{name}: ({x}, {y})");
            }
        }
    }

    #[test]
    fn empty_sketch_is_background_only() {
        let img = plot(&Sketch::new(), &opts()).unwrap();
        assert_eq!((img.width, img.height), (W, H));
        assert!(img.pixels.iter().all(|&c| c == BACKGROUND));
    }

    #[test]
    fn single_point_is_centered_without_dividing_by_zero() {
        let mut s = Sketch::new();
        s.add(Geometry::Point(Point::new(3.0, 4.0).unwrap()))
            .unwrap();
        let img = plot(&s, &opts()).unwrap();
        assert_eq!(img.get(W / 2, H / 2), DOT);
    }

    #[test]
    fn bad_size_is_an_error() {
        let s = Sketch::new();
        for (width, height) in [(0, 10), (10, 0), (MAX_SIDE + 1, 10)] {
            assert!(plot(&s, &PlotOptions { width, height }).is_err());
        }
    }

    #[test]
    fn circles_arcs_and_their_labels_are_drawn() {
        let mut s = Sketch::new();
        let p = |x, y| Point::new(x, y).unwrap();
        let circle = s
            .add(Geometry::Circle(Circle::new(p(0.0, 0.0), 10.0).unwrap()))
            .unwrap();
        // Upper half-arc, counter-clockwise from 0 to π.
        let arc = s
            .add(Geometry::Arc(
                Arc::new(p(30.0, 0.0), 10.0, 0.0, std::f64::consts::PI).unwrap(),
            ))
            .unwrap();
        s.add(Geometry::Point(p(15.0, -10.0))).unwrap();
        s.add_constraint(Constraint::Radius {
            geom: circle,
            value: 10.0,
        })
        .unwrap();
        s.add_constraint(Constraint::Diameter {
            geom: arc,
            value: 20.0,
        })
        .unwrap();
        let img = plot(&s, &opts()).unwrap();
        let fit = Fit::new([-10.0, -10.0], [40.0, 10.0], W, H);
        assert_eq!(
            bounds(&entities(&s).unwrap()),
            Some(([-10.0, -10.0], [40.0, 10.0]))
        );
        // Circle: drawn at its leftmost, bottom and top points.
        for (x, y) in [(-10.0, 0.0), (0.0, -10.0), (0.0, 10.0)] {
            assert!(drawn_near(&img, fit.pixel(x, y), 1, Some(GEOMETRY)) > 0);
        }
        // Arc: top is drawn, bottom (not swept) is not.
        assert!(drawn_near(&img, fit.pixel(30.0, 10.0), 1, Some(GEOMETRY)) > 0);
        assert_eq!(drawn_near(&img, fit.pixel(30.0, -10.0), 3, None), 0);
        // Point entity dot.
        assert!(drawn_near(&img, fit.pixel(15.0, -10.0), 0, Some(DOT)) > 0);
        // Labels: R at the circle's 45° point, Ø at the arc's top.
        let d = 10.0 * std::f64::consts::FRAC_1_SQRT_2;
        assert!(drawn_near(&img, fit.pixel(d, d), 25, Some(LABEL)) > 0);
        assert!(drawn_near(&img, fit.pixel(30.0, 10.0), 25, Some(LABEL)) > 0);
    }
}
