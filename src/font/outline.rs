use anyhow::{anyhow, bail, Result};

use crate::font::grammar::{Glyph, TrueTypeFontFile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    #[must_use]
    pub const fn mid(self, other: Point) -> Point {
        Point {
            x: f32::midpoint(self.x, other.x),
            y: f32::midpoint(self.y, other.y),
        }
    }
}

impl From<Point> for [f32; 2] {
    fn from(val: Point) -> Self {
        [val.x, val.y]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Segment {
    Line { p0: Point, p1: Point },
    QuadraticBezier { p0: Point, p1: Point, p2: Point },
}

impl Segment {
    pub fn start(self) -> Point {
        match self {
            Segment::Line { p0, .. } | Segment::QuadraticBezier { p0, .. } => p0,
        }
    }

    pub fn end(self) -> Point {
        match self {
            Segment::Line { p1, .. } => p1,
            Segment::QuadraticBezier { p2, .. } => p2,
        }
    }
}

pub type Contour = Vec<Segment>;

pub fn outline(font: &TrueTypeFontFile, glyph_id: u16) -> Result<Vec<Contour>> {
    let g = &font.glyph_table.glyphs[glyph_id as usize];

    match g {
        Glyph::Simple(simple_glyph) => {
            let mut contours = Vec::new();

            let points = simple_glyph
                .coordinates
                .iter()
                .zip(&simple_glyph.flags)
                .map(|(&(x, y), f)| {
                    (
                        Point {
                            x: x as f32,
                            y: y as f32,
                        },
                        f.on_curve(),
                    )
                })
                .collect::<Vec<_>>();

            let mut start = 0;

            for &end in &simple_glyph.end_points_of_contours {
                let end = end as usize;

                let contour = points
                    .get(start..=end)
                    .ok_or_else(|| anyhow!("contour end point {end} out of range"))?;

                contours.push(walk_contour(contour));
                start = end + 1;
            }

            Ok(contours)
        }
        Glyph::Compound(_compound_glyph) => bail!("compound glyphs are not supported (yet)"),
        Glyph::Empty => Ok(vec![]),
    }
}

fn walk_contour(points: &[(Point, bool)]) -> Contour {
    if points.is_empty() {
        return vec![];
    }

    let n = points.len();

    let (start, rest) = {
        let first = points.first().expect("points is nonempty");
        let last = points.last().expect("points is nonempty");

        if first.1 {
            (first.0, &points[1..])
        } else if last.1 {
            (last.0, &points[..n - 1])
        } else {
            (first.0.mid(last.0), points)
        }
    };

    let mut pen = start;
    let mut ctrl = None;
    let mut out = Vec::with_capacity(points.len() + 1);

    for &(p, on_curve) in rest.iter().chain(std::iter::once(&(start, true))) {
        match (on_curve, ctrl) {
            (true, None) => {
                if p != pen {
                    out.push(Segment::Line { p0: pen, p1: p });
                }

                pen = p;
            }
            (true, Some(c)) => {
                out.push(Segment::QuadraticBezier {
                    p0: pen,
                    p1: c,
                    p2: p,
                });

                pen = p;
                ctrl = None;
            }
            (false, None) => {
                ctrl = Some(p);
            }
            (false, Some(c)) => {
                let mid = c.mid(p);
                out.push(Segment::QuadraticBezier {
                    p0: pen,
                    p1: c,
                    p2: mid,
                });

                pen = mid;
                ctrl = Some(p);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::TrueTypeFontParser;
    use std::fs;

    const ON: bool = true;
    const OFF: bool = false;

    const fn p(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    fn pts(v: &[(f32, f32, bool)]) -> Vec<(Point, bool)> {
        v.iter().map(|&(x, y, on)| (p(x, y), on)).collect()
    }

    const fn line(p0: Point, p1: Point) -> Segment {
        Segment::Line { p0, p1 }
    }

    const fn quad(p0: Point, p1: Point, p2: Point) -> Segment {
        Segment::QuadraticBezier { p0, p1, p2 }
    }

    #[test]
    fn walk_square() {
        let contour = walk_contour(&pts(&[
            (0.0, 0.0, ON),
            (10.0, 0.0, ON),
            (10.0, 10.0, ON),
            (0.0, 10.0, ON),
        ]));

        assert_eq!(
            contour,
            [
                line(p(0.0, 0.0), p(10.0, 0.0)),
                line(p(10.0, 0.0), p(10.0, 10.0)),
                line(p(10.0, 10.0), p(0.0, 10.0)),
                line(p(0.0, 10.0), p(0.0, 0.0)),
            ]
        );
    }

    const DOME: [Segment; 3] = [
        quad(p(0.0, 0.0), p(0.0, 100.0), p(50.0, 100.0)),
        quad(p(50.0, 100.0), p(100.0, 100.0), p(100.0, 0.0)),
        line(p(100.0, 0.0), p(0.0, 0.0)),
    ];

    #[test]
    fn walk_dome() {
        // two consecutive off-curve points imply an on-curve point at (50, 100)
        let contour = walk_contour(&pts(&[
            (0.0, 0.0, ON),
            (0.0, 100.0, OFF),
            (100.0, 100.0, OFF),
            (100.0, 0.0, ON),
        ]));

        assert_eq!(contour, DOME);
    }

    #[test]
    fn walk_dome_starting_off_curve() {
        // same contour rotated so it starts off-curve; the walk starts from the last point
        let contour = walk_contour(&pts(&[
            (0.0, 100.0, OFF),
            (100.0, 100.0, OFF),
            (100.0, 0.0, ON),
            (0.0, 0.0, ON),
        ]));

        assert_eq!(contour, DOME);
    }

    #[test]
    fn walk_all_off_curve() {
        // no real on-curve points, so the walk starts at mid(first, last)
        let contour = walk_contour(&pts(&[
            (0.0, 1.0, OFF),
            (1.0, 0.0, OFF),
            (0.0, -1.0, OFF),
            (-1.0, 0.0, OFF),
        ]));

        assert_eq!(
            contour,
            [
                quad(p(-0.5, 0.5), p(0.0, 1.0), p(0.5, 0.5)),
                quad(p(0.5, 0.5), p(1.0, 0.0), p(0.5, -0.5)),
                quad(p(0.5, -0.5), p(0.0, -1.0), p(-0.5, -0.5)),
                quad(p(-0.5, -0.5), p(-1.0, 0.0), p(-0.5, 0.5)),
            ]
        );
    }

    #[test]
    fn every_simple_glyph_outline_is_closed() {
        let ttf_file = fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/src/font/Lato-Regular.ttf")).unwrap();
        let ttf = TrueTypeFontParser::new(&ttf_file).parse().unwrap();

        for (id, glyph) in ttf.glyph_table.glyphs.iter().enumerate() {
            // compound glyphs are not supported yet
            let Glyph::Simple(simple) = glyph else {
                continue;
            };

            let contours = outline(&ttf, id as u16).unwrap();

            assert_eq!(contours.len(), simple.end_points_of_contours.len(),);

            for contour in &contours {
                assert!(!contour.is_empty(), "contour should be nonempty");

                for (i, seg) in contour.iter().enumerate() {
                    let next = contour[(i + 1) % contour.len()];
                    assert_eq!(seg.end(), next.start(),);
                }
            }
        }
    }
}
