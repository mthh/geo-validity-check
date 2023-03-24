mod coord;
mod geometry;
mod geometrycollection;
mod line;
mod linestring;
mod multilinestring;
mod multipoint;
mod multipolygon;
mod point;
mod polygon;
mod rect;
mod triangle;
mod utils;

use std::boxed::Box;
use std::fmt::Display;

#[derive(Debug, PartialEq)]
/// The role of a ring in a polygon.
pub enum RingRole {
    Exterior,
    Interior(usize),
}

#[derive(Debug, PartialEq)]
/// The position of the problem in a multi-geometry.
pub struct GeometryPosition(usize);

#[derive(Debug, PartialEq)]
/// The coordinate position of the problem in the geometry.
pub struct CoordinatePosition(isize);

#[derive(Debug, PartialEq)]
/// The position of the problem in the geometry.
pub enum ProblemPosition {
    Point,
    Line(CoordinatePosition),
    Triangle(CoordinatePosition),
    Rect(CoordinatePosition),
    MultiPoint(GeometryPosition),
    LineString(CoordinatePosition),
    MultiLineString(GeometryPosition, CoordinatePosition),
    Polygon(RingRole, CoordinatePosition),
    MultiPolygon(GeometryPosition, RingRole, CoordinatePosition),
    GeometryCollection(GeometryPosition, Box<ProblemPosition>),
}

#[derive(Debug, PartialEq)]
/// The type of problem encountered.
pub enum Problem {
    /// A coordinate is not finite (NaN or infinite)
    NotFinite,
    /// A LineString or a Polygon ring has too few points
    TooFewPoints,
    /// Identical coords
    IdenticalCoords,
    /// Collinear coords
    CollinearCoords,
    /// A ring has a self-intersection
    SelfIntersection,
    /// Two interior rings of a Polygon share a common line
    IntersectingRingsOnALine,
    /// Two interior rings of a Polygon share a common area
    IntersectingRingsOnAnArea,
    /// The interior ring of a Polygon is not contained in the exterior ring
    InteriorRingNotContainedInExteriorRing,
    /// Two Polygons of MultiPolygons overlap partially
    ElementsOverlaps,
    /// Two Polygons of MultiPolygons touch on a line
    ElementsTouchOnALine,
    /// Two Polygons of MultiPolygons are identical
    ElementsAreIdentical,
}

#[derive(Debug, PartialEq)]
/// A problem, at a given position, encountered when checking the validity of a geometry.
pub struct ProblemAtPosition(Problem, ProblemPosition);

impl Display for ProblemAtPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at {:?}", self.0, self.1)
    }
}

/// All the problems encountered when checking the validity of a geometry.
pub struct ProblemReport(Vec<ProblemAtPosition>);

/// A trait to check if a geometry is valid and report the reason(s) of invalidity.
pub trait Valid {
    /// Check if the geometry is valid.
    fn is_valid(&self) -> bool;
    /// Return the reason(s) of invalidity, or None if valid
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>>;
}
