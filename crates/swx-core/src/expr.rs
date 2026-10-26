//! Parameter expressions with units, e.g. `d1 / 2 + 5 mm`.
//!
//! [`parse`] turns source text into an [`Expr`]; [`eval`] computes a
//! [`Quantity`] with dimensional checking, looking names up through a
//! caller-supplied resolver. Values are kept in base units (millimeters and
//! radians); [`Quantity::in_unit`] converts back for display.
//!
//! Grammar (usual precedence, left-associative binary operators):
//!
//! ```text
//! expr    := term (('+' | '-') term)*
//! term    := unary (('*' | '/') unary)*
//! unary   := '-' unary | postfix
//! postfix := number [unit] | '(' expr ')' [unit] | ident
//! ```
//!
//! A unit suffix is an identifier directly after a number or a closing
//! parenthesis, so `2 in` is two inches while `in + 1` refers to a parameter
//! named `in`. A suffix multiplies by the unit: `(2 + 3) mm` is 5 mm.

use std::fmt;

/// Maximum nesting of parentheses and unary minus. Keeps recursion bounded.
const MAX_DEPTH: usize = 64;
/// Maximum number of tokens in one expression. Bounds the tree size, and so
/// the recursion depth of evaluation, name collection and drop.
const MAX_TOKENS: usize = 1024;

/// Physical dimension as integer exponents of length and angle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dim {
    pub length: i32,
    pub angle: i32,
}

impl Dim {
    pub const NONE: Dim = Dim::new(0, 0);
    pub const LENGTH: Dim = Dim::new(1, 0);
    pub const ANGLE: Dim = Dim::new(0, 1);

    pub const fn new(length: i32, angle: i32) -> Dim {
        Dim { length, angle }
    }

    /// Dimension of a product (`sign = 1`) or quotient (`sign = -1`).
    fn combine(self, other: Dim, sign: i32) -> Dim {
        Dim::new(
            self.length + sign * other.length,
            self.angle + sign * other.angle,
        )
    }
}

impl fmt::Display for Dim {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        for (name, exp) in [("length", self.length), ("angle", self.angle)] {
            match exp {
                0 => {}
                1 => parts.push(name.to_owned()),
                e => parts.push(format!("{name}^{e}")),
            }
        }
        if parts.is_empty() {
            f.write_str("unitless")
        } else {
            f.write_str(&parts.join("*"))
        }
    }
}

/// A value in base units (millimeters, radians) with its dimension.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub dim: Dim,
}

impl Quantity {
    pub fn new(value: f64, dim: Dim) -> Quantity {
        Quantity { value, dim }
    }

    /// The value expressed in `unit`, for display. The base value is divided
    /// by the unit's factor raised to the matching exponent of `dim` (length
    /// units use the length exponent, angle units the angle exponent), so
    /// 25.4 mm is 1.0 in and 645.16 mm² is 1.0 in². A dimension without that
    /// base (e.g. unitless) is returned unchanged.
    pub fn in_unit(&self, unit: Unit) -> f64 {
        let exp = if unit.dim() == Dim::LENGTH {
            self.dim.length
        } else {
            self.dim.angle
        };
        self.value / unit.factor().powi(exp)
    }
}

/// Units accepted as suffixes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unit {
    Mm,
    Cm,
    M,
    In,
    Ft,
    Deg,
    Rad,
}

impl Unit {
    /// Look up a unit by its suffix spelling (`mm`, `cm`, `m`, `in`, `ft`, `deg`, `rad`).
    pub fn from_suffix(s: &str) -> Option<Unit> {
        Some(match s {
            "mm" => Unit::Mm,
            "cm" => Unit::Cm,
            "m" => Unit::M,
            "in" => Unit::In,
            "ft" => Unit::Ft,
            "deg" => Unit::Deg,
            "rad" => Unit::Rad,
            _ => return None,
        })
    }

    /// Size of one unit in base units (millimeters or radians).
    pub fn factor(self) -> f64 {
        match self {
            Unit::Mm => 1.0,
            Unit::Cm => 10.0,
            Unit::M => 1000.0,
            Unit::In => 25.4,
            Unit::Ft => 304.8,
            Unit::Deg => std::f64::consts::PI / 180.0,
            Unit::Rad => 1.0,
        }
    }

    pub fn dim(self) -> Dim {
        match self {
            Unit::Deg | Unit::Rad => Dim::ANGLE,
            _ => Dim::LENGTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// A parsed expression. Offsets are byte offsets into the source text.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Name {
        name: String,
        offset: usize,
    },
    /// `inner` followed by a unit suffix: `inner` times one `unit`.
    WithUnit {
        inner: Box<Expr>,
        unit: Unit,
    },
    Neg(Box<Expr>),
    Bin {
        op: BinOp,
        offset: usize,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

impl Expr {
    /// Identifiers this expression references, in first-use order, without duplicates.
    pub fn names(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.collect_names(&mut out);
        out
    }

    fn collect_names(&self, out: &mut Vec<String>) {
        match self {
            Expr::Num(_) => {}
            Expr::Name { name, .. } => {
                if !out.contains(name) {
                    out.push(name.clone());
                }
            }
            Expr::WithUnit { inner, .. } | Expr::Neg(inner) => inner.collect_names(out),
            Expr::Bin { lhs, rhs, .. } => {
                lhs.collect_names(out);
                rhs.collect_names(out);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprErrorKind {
    Syntax(String),
    UnknownUnit(String),
    UnknownName(String),
    DimensionMismatch { op: char, left: Dim, right: Dim },
    DivisionByZero,
}

/// An expression error with the byte offset in the source where it applies.
#[derive(Debug, Clone, PartialEq)]
pub struct ExprError {
    pub kind: ExprErrorKind,
    pub offset: usize,
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let at = self.offset;
        match &self.kind {
            ExprErrorKind::Syntax(msg) => write!(f, "syntax error at byte {at}: {msg}"),
            ExprErrorKind::UnknownUnit(u) => write!(
                f,
                "unknown unit `{u}` at byte {at} (expected mm, cm, m, in, ft, deg or rad)"
            ),
            ExprErrorKind::UnknownName(n) => write!(f, "unknown name `{n}` at byte {at}"),
            ExprErrorKind::DimensionMismatch { op, left, right } => write!(
                f,
                "dimension mismatch at byte {at}: cannot apply `{op}` to {left} and {right}"
            ),
            ExprErrorKind::DivisionByZero => write!(f, "division by zero at byte {at}"),
        }
    }
}

impl std::error::Error for ExprError {}

fn err<T>(kind: ExprErrorKind, offset: usize) -> Result<T, ExprError> {
    Err(ExprError { kind, offset })
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Ident(String),
    Op(char),
}

fn lex(src: &str) -> Result<Vec<(Tok, usize)>, ExprError> {
    let bytes = src.as_bytes();
    let digit_at = |i: usize| bytes.get(i).is_some_and(u8::is_ascii_digit);
    let mut toks = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        let start = i;
        if c.is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || (c == b'.' && digit_at(i + 1)) {
            while digit_at(i) {
                i += 1;
            }
            if bytes.get(i) == Some(&b'.') {
                i += 1;
                while digit_at(i) {
                    i += 1;
                }
            }
            // Exponent only if digits follow, so `5em` lexes as `5` then `em`.
            if matches!(bytes.get(i), Some(b'e' | b'E')) {
                let sign = usize::from(matches!(bytes.get(i + 1), Some(b'+' | b'-')));
                if digit_at(i + 1 + sign) {
                    i += 1 + sign;
                    while digit_at(i) {
                        i += 1;
                    }
                }
            }
            match src[start..i].parse::<f64>() {
                Ok(v) if v.is_finite() => toks.push((Tok::Num(v), start)),
                _ => return err(ExprErrorKind::Syntax("number out of range".into()), start),
            }
        } else if c.is_ascii_alphabetic() || c == b'_' {
            while bytes
                .get(i)
                .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
            {
                i += 1;
            }
            toks.push((Tok::Ident(src[start..i].to_owned()), start));
        } else if matches!(c, b'+' | b'-' | b'*' | b'/' | b'(' | b')') {
            toks.push((Tok::Op(c as char), start));
            i += 1;
        } else {
            let ch = src[start..].chars().next().unwrap_or('?');
            return err(
                ExprErrorKind::Syntax(format!("unexpected character `{ch}`")),
                start,
            );
        }
        if toks.len() > MAX_TOKENS {
            return err(ExprErrorKind::Syntax("expression too long".into()), start);
        }
    }
    Ok(toks)
}

/// Parse an expression.
///
/// Hand-written recursive-descent parser with one function per precedence
/// level, following Aho, Lam, Sethi & Ullman, *Compilers: Principles,
/// Techniques, and Tools*, 2nd ed., §2.4 (predictive parsing) and §4.4
/// (top-down parsing, left-recursion elimination giving left associativity).
pub fn parse(src: &str) -> Result<Expr, ExprError> {
    let mut p = Parser {
        toks: lex(src)?,
        pos: 0,
        depth: 0,
        end: src.len(),
    };
    let e = p.expr()?;
    match p.peek() {
        None => Ok(e),
        Some((t, off)) => err(
            ExprErrorKind::Syntax(format!("unexpected {}", describe(t))),
            off,
        ),
    }
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Num(_) => "number".into(),
        Tok::Ident(s) => format!("name `{s}`"),
        Tok::Op(c) => format!("`{c}`"),
    }
}

struct Parser {
    toks: Vec<(Tok, usize)>,
    pos: usize,
    depth: usize,
    end: usize,
}

impl Parser {
    fn peek(&self) -> Option<(&Tok, usize)> {
        self.toks.get(self.pos).map(|(t, o)| (t, *o))
    }

    fn eat_op(&mut self, ops: &[char]) -> Option<(char, usize)> {
        match self.peek() {
            Some((Tok::Op(c), off)) if ops.contains(c) => {
                let c = *c;
                self.pos += 1;
                Some((c, off))
            }
            _ => None,
        }
    }

    fn binary(
        &mut self,
        ops: &[char],
        next: fn(&mut Self) -> Result<Expr, ExprError>,
    ) -> Result<Expr, ExprError> {
        let mut lhs = next(self)?;
        while let Some((c, offset)) = self.eat_op(ops) {
            let rhs = next(self)?;
            let op = match c {
                '+' => BinOp::Add,
                '-' => BinOp::Sub,
                '*' => BinOp::Mul,
                _ => BinOp::Div,
            };
            lhs = Expr::Bin {
                op,
                offset,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn expr(&mut self) -> Result<Expr, ExprError> {
        self.binary(&['+', '-'], Self::term)
    }

    fn term(&mut self) -> Result<Expr, ExprError> {
        self.binary(&['*', '/'], Self::unary)
    }

    fn enter(&mut self, offset: usize) -> Result<(), ExprError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return err(
                ExprErrorKind::Syntax("expression nested too deeply".into()),
                offset,
            );
        }
        Ok(())
    }

    fn unary(&mut self) -> Result<Expr, ExprError> {
        if let Some((_, off)) = self.eat_op(&['-']) {
            self.enter(off)?;
            let inner = self.unary()?;
            self.depth -= 1;
            return Ok(Expr::Neg(Box::new(inner)));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, ExprError> {
        let Some((tok, off)) = self.peek() else {
            return err(
                ExprErrorKind::Syntax("expected a number, name or `(`".into()),
                self.end,
            );
        };
        let tok = tok.clone();
        self.pos += 1;
        let base = match tok {
            Tok::Num(v) => Expr::Num(v),
            Tok::Ident(name) => return Ok(Expr::Name { name, offset: off }),
            Tok::Op('(') => {
                self.enter(off)?;
                let inner = self.expr()?;
                self.depth -= 1;
                if self.eat_op(&[')']).is_none() {
                    let (msg, at) = match self.peek() {
                        Some((t, o)) => (format!("expected `)`, found {}", describe(t)), o),
                        None => ("expected `)`".to_owned(), self.end),
                    };
                    return err(ExprErrorKind::Syntax(msg), at);
                }
                inner
            }
            t => {
                return err(
                    ExprErrorKind::Syntax(format!(
                        "expected a number, name or `(`, found {}",
                        describe(&t)
                    )),
                    off,
                )
            }
        };
        if let Some((Tok::Ident(s), uoff)) = self.peek() {
            let Some(unit) = Unit::from_suffix(s) else {
                return err(ExprErrorKind::UnknownUnit(s.clone()), uoff);
            };
            self.pos += 1;
            return Ok(Expr::WithUnit {
                inner: Box::new(base),
                unit,
            });
        }
        Ok(base)
    }
}

/// Evaluate `expr`, looking names up with `resolve`.
///
/// A bare number is unitless; the caller decides how to treat a unitless
/// result. The result is always in base units (mm, rad), so it does not
/// depend on `default_length`: that argument is the document's display unit
/// for lengths, passed through so callers hand over the document setting, and
/// is meant for showing the result with [`Quantity::in_unit`]. Evaluation
/// itself does not read it.
pub fn eval(
    expr: &Expr,
    resolve: &dyn Fn(&str) -> Option<Quantity>,
    default_length: Unit,
) -> Result<Quantity, ExprError> {
    let _display_unit = default_length;
    eval_inner(expr, resolve)
}

fn eval_inner(
    expr: &Expr,
    resolve: &dyn Fn(&str) -> Option<Quantity>,
) -> Result<Quantity, ExprError> {
    Ok(match expr {
        Expr::Num(v) => Quantity::new(*v, Dim::NONE),
        Expr::Name { name, offset } => match resolve(name) {
            Some(q) => q,
            None => return err(ExprErrorKind::UnknownName(name.clone()), *offset),
        },
        Expr::WithUnit { inner, unit } => {
            let q = eval_inner(inner, resolve)?;
            Quantity::new(q.value * unit.factor(), q.dim.combine(unit.dim(), 1))
        }
        Expr::Neg(inner) => {
            let q = eval_inner(inner, resolve)?;
            Quantity::new(-q.value, q.dim)
        }
        Expr::Bin {
            op,
            offset,
            lhs,
            rhs,
        } => {
            let a = eval_inner(lhs, resolve)?;
            let b = eval_inner(rhs, resolve)?;
            let mismatch = |c| {
                err(
                    ExprErrorKind::DimensionMismatch {
                        op: c,
                        left: a.dim,
                        right: b.dim,
                    },
                    *offset,
                )
            };
            match op {
                BinOp::Add | BinOp::Sub if a.dim != b.dim => {
                    return mismatch(if *op == BinOp::Add { '+' } else { '-' })
                }
                BinOp::Add => Quantity::new(a.value + b.value, a.dim),
                BinOp::Sub => Quantity::new(a.value - b.value, a.dim),
                BinOp::Mul => Quantity::new(a.value * b.value, a.dim.combine(b.dim, 1)),
                BinOp::Div if b.value == 0.0 => return err(ExprErrorKind::DivisionByZero, *offset),
                BinOp::Div => Quantity::new(a.value / b.value, a.dim.combine(b.dim, -1)),
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn params(name: &str) -> Option<Quantity> {
        match name {
            "d1" => Some(Quantity::new(40.0, Dim::LENGTH)),
            "w" => Some(Quantity::new(3.0, Dim::LENGTH)),
            "a" => Some(Quantity::new(PI / 4.0, Dim::ANGLE)),
            "in" => Some(Quantity::new(7.0, Dim::NONE)),
            _ => None,
        }
    }

    fn run(src: &str) -> Result<Quantity, ExprError> {
        eval(&parse(src)?, &params, Unit::Mm)
    }

    #[test]
    fn valid_expressions() {
        let l2 = Dim::new(2, 0);
        let cases: &[(&str, f64, Dim, f64)] = &[
            ("5", 5.0, Dim::NONE, 0.0),
            ("2.5", 2.5, Dim::NONE, 0.0),
            (".5", 0.5, Dim::NONE, 0.0),
            ("1e-3", 0.001, Dim::NONE, 0.0),
            ("2.5E+2", 250.0, Dim::NONE, 0.0),
            ("5 mm", 5.0, Dim::LENGTH, 0.0),
            ("5mm", 5.0, Dim::LENGTH, 0.0),
            ("1 cm", 10.0, Dim::LENGTH, 0.0),
            ("0.25 m", 250.0, Dim::LENGTH, 0.0),
            ("1 in", 25.4, Dim::LENGTH, 0.0),
            ("2 ft + 1 in", 635.0, Dim::LENGTH, 1e-9),
            ("90 deg", PI / 2.0, Dim::ANGLE, 1e-12),
            ("1 rad", 1.0, Dim::ANGLE, 0.0),
            ("d1 / 2 + 5 mm", 25.0, Dim::LENGTH, 1e-12),
            ("d1 * d1", 1600.0, l2, 0.0),
            ("d1 * d1 / w", 40.0 * 40.0 / 3.0, Dim::LENGTH, 1e-12),
            ("(d1 + 2 mm) / 2", 21.0, Dim::LENGTH, 0.0),
            ("(2 + 3) mm", 5.0, Dim::LENGTH, 0.0),
            ("(2 + 3)in", 127.0, Dim::LENGTH, 1e-12),
            ("-(-1 in)", 25.4, Dim::LENGTH, 0.0),
            ("a * 2", PI / 2.0, Dim::ANGLE, 1e-12),
            ("in + 1", 8.0, Dim::NONE, 0.0),
            ("2 in * in", 25.4 * 14.0, Dim::LENGTH, 1e-9),
            ("d1 / w", 40.0 / 3.0, Dim::NONE, 1e-12),
        ];
        for &(src, value, dim, tol) in cases {
            let q = run(src).unwrap_or_else(|e| panic!("{src}: {e}"));
            assert!(
                (q.value - value).abs() <= tol,
                "{src}: got {}, want {value}",
                q.value
            );
            assert_eq!(q.dim, dim, "{src}");
        }
        assert_eq!(run("1 in").unwrap().value, 25.4);
    }

    #[test]
    fn precedence_and_associativity() {
        for (src, want) in [
            ("2 + 3 * 4", 14.0),
            ("8 / 4 / 2", 1.0),
            ("-2 * 3", -6.0),
            ("10 - 4 - 3", 3.0),
            ("(2 + 3) * 4", 20.0),
            ("2 * -3", -6.0),
            ("--2", 2.0),
            ("1 + 2 * 3 - 4 / 2", 5.0),
        ] {
            assert_eq!(run(src).unwrap().value, want, "{src}");
        }
    }

    #[test]
    fn unit_conversion_for_display() {
        let q = run("25.4 mm").unwrap();
        assert_eq!(q.in_unit(Unit::In), 1.0);
        assert!((run("1 ft").unwrap().in_unit(Unit::In) - 12.0).abs() < 1e-12);
        assert!((run("2 m").unwrap().in_unit(Unit::Cm) - 200.0).abs() < 1e-12);
        assert!((run("3.14159265358979 rad").unwrap().in_unit(Unit::Deg) - 180.0).abs() < 1e-9);
        let area = run("1 in * 1 in").unwrap();
        assert!((area.in_unit(Unit::In) - 1.0).abs() < 1e-12);
        assert_eq!(run("3").unwrap().in_unit(Unit::Ft), 3.0);
        // `default_length` affects display only, never the evaluated value.
        let e = parse("d1 / 2 + 5 mm").unwrap();
        assert_eq!(eval(&e, &params, Unit::In), eval(&e, &params, Unit::Mm));
    }

    #[test]
    fn invalid_expressions() {
        use ExprErrorKind as K;
        type Check = fn(&K) -> bool;
        let cases: &[(&str, Check, usize)] = &[
            ("2 +", |k| matches!(k, K::Syntax(_)), 3),
            ("(1", |k| matches!(k, K::Syntax(_)), 2),
            ("1 2", |k| matches!(k, K::Syntax(_)), 2),
            ("5 mmm", |k| *k == K::UnknownUnit("mmm".into()), 2),
            ("5 kg", |k| *k == K::UnknownUnit("kg".into()), 2),
            ("x + 1", |k| *k == K::UnknownName("x".into()), 0),
            ("1 + foo", |k| *k == K::UnknownName("foo".into()), 4),
            (
                "5 mm + 2 deg",
                |k| matches!(k, K::DimensionMismatch { op: '+', .. }),
                5,
            ),
            ("d1 + 3", |k| matches!(k, K::DimensionMismatch { .. }), 3),
            (
                "d1 - a",
                |k| matches!(k, K::DimensionMismatch { op: '-', .. }),
                3,
            ),
            ("1 / 0", |k| *k == K::DivisionByZero, 2),
            ("d1 / (w - 3 mm)", |k| *k == K::DivisionByZero, 3),
            ("", |k| matches!(k, K::Syntax(_)), 0),
            ("   ", |k| matches!(k, K::Syntax(_)), 3),
            (")", |k| matches!(k, K::Syntax(_)), 0),
            ("()", |k| matches!(k, K::Syntax(_)), 1),
            ("(1))", |k| matches!(k, K::Syntax(_)), 3),
            ("1 + * 2", |k| matches!(k, K::Syntax(_)), 4),
            ("*2", |k| matches!(k, K::Syntax(_)), 0),
            ("2 ^ 3", |k| matches!(k, K::Syntax(_)), 2),
            ("1..2", |k| matches!(k, K::Syntax(_)), 2),
            (".", |k| matches!(k, K::Syntax(_)), 0),
            ("d1 mm", |k| matches!(k, K::Syntax(_)), 3),
            ("5 mm mm", |k| matches!(k, K::Syntax(_)), 5),
            ("2 (3)", |k| matches!(k, K::Syntax(_)), 2),
            ("1e999", |k| matches!(k, K::Syntax(_)), 0),
            ("5 µm", |k| matches!(k, K::Syntax(_)), 2),
            ("1 + 2 -", |k| matches!(k, K::Syntax(_)), 7),
            ("5em", |k| *k == K::UnknownUnit("em".into()), 1),
            ("1 +\t#", |k| matches!(k, K::Syntax(_)), 4),
        ];
        assert!(cases.len() >= 20);
        for &(src, check, offset) in cases {
            let e = run(src).expect_err(src);
            assert!(check(&e.kind), "{src}: wrong kind {e:?}");
            assert_eq!(e.offset, offset, "{src}: {e}");
            assert!(!e.to_string().is_empty());
        }
    }

    #[test]
    fn no_panic_on_pathological_input() {
        for src in [
            "(".repeat(10_000),
            "-".repeat(10_000) + "1",
            "1+".repeat(10_000) + "1",
            ")".repeat(100),
            "\u{0}\u{7f}é".into(),
        ] {
            assert!(run(&src).is_err());
        }
        // Moderate nesting still works.
        let src = "(".repeat(50) + "1" + &")".repeat(50);
        assert_eq!(run(&src).unwrap().value, 1.0);
    }

    #[test]
    fn error_messages() {
        assert_eq!(
            run("5 mm + 2 deg").unwrap_err().to_string(),
            "dimension mismatch at byte 5: cannot apply `+` to length and angle"
        );
        assert_eq!(
            run("d1 + 3").unwrap_err().to_string(),
            "dimension mismatch at byte 3: cannot apply `+` to length and unitless"
        );
        assert_eq!(
            run("x").unwrap_err().to_string(),
            "unknown name `x` at byte 0"
        );
        assert_eq!(
            run("1/0").unwrap_err().to_string(),
            "division by zero at byte 1"
        );
        assert!(run("5 mmm")
            .unwrap_err()
            .to_string()
            .contains("unknown unit `mmm`"));
        assert_eq!(Dim::new(2, -1).to_string(), "length^2*angle^-1");
    }

    #[test]
    fn names_in_first_use_order() {
        let e = parse("d1 / 2 + d1 * w").unwrap();
        assert_eq!(e.names(), vec!["d1", "w"]);
        assert_eq!(parse("-(b + a) mm * b").unwrap().names(), vec!["b", "a"]);
        assert!(parse("2 in + 3").unwrap().names().is_empty());
    }
}
