mod tests;
mod utils;

use geo::coordinate_position::CoordPos;
use geo::dimensions::Dimensions;
use geo::{Contains, RemoveRepeatedPoints};
use geo::{Intersects, Relate};
use geo_types::{
    Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};
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
pub struct CoordinatePosition(usize);

#[derive(Debug, PartialEq)]
/// The position of the problem in the geometry.
pub enum ProblemPosition {
    Point,
    MultiPoint(GeometryPosition),
    LineString(CoordinatePosition),
    MultiLineString(GeometryPosition, CoordinatePosition),
    Polygon(RingRole, CoordinatePosition),
    MultiPolygon(GeometryPosition, RingRole, CoordinatePosition),
}

#[derive(Debug, PartialEq)]
/// The type of problem encountered.
pub enum Problem {
    NotFinite,
    TooFewPoints,
    SelfIntersection,
    IntersectingRingsOnALine,
    IntersectingRingsOnAnArea,
    InteriorRingNotContainedInExteriorRing,
    ElementsOverlaps,
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

impl Valid for Coord {
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(&self) {
            return false;
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(&self) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point,
            ));
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

/// In PostGIS, Point don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl Valid for Point {
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        self.0.invalidity_reason()
    }
}

/// In PostGIS, MultiPoint don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl Valid for MultiPoint {
    fn is_valid(&self) -> bool {
        for point in &self.0 {
            if !point.is_valid() {
                return false;
            }
        }
        true
    }

    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        for (i, point) in self.0.iter().enumerate() {
            if utils::check_coord_is_not_finite(&point.0) {
                reason.push(ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(i)),
                ));
            }
        }

        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

/// In postGIS, a LineString is valid if it has at least 2 points
/// and have a non-zero length (i.e. the first and last points are not the same).
/// Here we also check that all its points are finite numbers.
impl Valid for LineString {
    fn is_valid(&self) -> bool {
        if utils::check_too_few_points(self, false) {
            return false;
        }
        for coord in &self.0 {
            if !coord.is_valid() {
                return false;
            }
        }
        true
    }

    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        // Perform the various checks
        if utils::check_too_few_points(self, false) {
            reason.push(ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::LineString(CoordinatePosition(0)),
            ));
        }

        for (i, point) in self.0.iter().enumerate() {
            if utils::check_coord_is_not_finite(point) {
                reason.push(ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::LineString(CoordinatePosition(i)),
                ));
            }
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

/// MultiLineString is valid if all its LineStrings are valid.
impl Valid for MultiLineString {
    fn is_valid(&self) -> bool {
        for line in &self.0 {
            if !line.is_valid() {
                return false;
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        for (j, line) in self.0.iter().enumerate() {
            let temp_reason = line.invalidity_reason();
            if let Some(temp_reason) = temp_reason {
                for ProblemAtPosition(problem, position) in temp_reason {
                    match position {
                        ProblemPosition::LineString(coord_pos) => {
                            reason.push(ProblemAtPosition(
                                problem,
                                ProblemPosition::MultiLineString(GeometryPosition(j), coord_pos),
                            ));
                        }
                        _ => unreachable!(),
                    }
                }
            }
            // // Perform the various checks
            // if check_too_few_points(line, false) {
            //     reason.push((
            //         Problem::TooFewPoints,
            //         ProblemPosition::MultiLineString(GeometryPosition(j), CoordinatePosition(0)),
            //     ));
            // }
            //
            // for (i, point) in line.0.iter().enumerate() {
            //     if check_coord_is_not_finite(point) {
            //         reason.push((
            //             Problem::NotFinite,
            //             ProblemPosition::MultiLineString(GeometryPosition(j), CoordinatePosition(i)),
            //         ));
            //     }
            // }
        }
        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

/// Polygon must follow the following rules to be valid:
/// - [x] the polygon boundary rings (the exterior shell ring and interior hole rings) are simple (do not cross or self-touch). Because of this a polygon cannnot have cut lines, spikes or loops. This implies that polygon holes must be represented as interior rings, rather than by the exterior ring self-touching (a so-called "inverted hole").
/// - [x] boundary rings do not cross
/// - [x] boundary rings may touch at points but only as a tangent (i.e. not in a line)
/// - [x] interior rings are contained in the exterior ring
/// - [ ] the polygon interior is simply connected (i.e. the rings must not touch in a way that splits the polygon into more than one part)
impl Valid for Polygon {
    fn is_valid(&self) -> bool {
        for ring in self.interiors().iter().chain([self.exterior()]) {
            if utils::check_too_few_points(ring, true) {
                return false;
            }
            for coord in ring {
                if !coord.is_valid() {
                    return false;
                }
            }
            if utils::linestring_has_self_intersection(ring) {
                return false;
            }
        }

        let polygon_exterior = Polygon::new(self.exterior().clone(), vec![]);

        for interior_ring in self.interiors() {
            // geo::contains::Contains return true if the interior
            // is contained in the exterior even if they touches on one or more points
            if !polygon_exterior.contains(interior_ring) {
                return false;
            }

            let im = polygon_exterior.relate(interior_ring);

            // Interior ring and exterior ring may only touch at point (not as a line)
            // and not cross
            match im.get(CoordPos::OnBoundary, CoordPos::Inside) {
                Dimensions::OneDimensional | Dimensions::TwoDimensional => {
                    return false;
                }
                _ => {}
            };

            let pol_interior1 = Polygon::new(interior_ring.clone(), vec![]);

            for (i, interior2) in self.interiors().iter().enumerate() {
                if interior_ring != interior2 {
                    let pol_interior2 = Polygon::new(interior2.clone(), vec![]);
                    let intersection_matrix = pol_interior1.relate(&pol_interior2);
                    match intersection_matrix.get(CoordPos::Inside, CoordPos::Inside) {
                        Dimensions::TwoDimensional => {
                            return false;
                        }
                        _ => {}
                    }
                    match intersection_matrix.get(CoordPos::OnBoundary, CoordPos::OnBoundary) {
                        Dimensions::OneDimensional => {
                            return false;
                        }
                        _ => {}
                    }
                }
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        for (j, ring) in self.interiors().iter().chain([self.exterior()]).enumerate() {
            // Perform the various checks
            if utils::check_too_few_points(ring, true) {
                reason.push(ProblemAtPosition(
                    Problem::TooFewPoints,
                    ProblemPosition::Polygon(
                        if j == 0 {
                            RingRole::Exterior
                        } else {
                            RingRole::Interior(j)
                        },
                        CoordinatePosition(0),
                    ),
                ));
            }

            if utils::linestring_has_self_intersection(ring) {
                reason.push(ProblemAtPosition(
                    Problem::SelfIntersection,
                    ProblemPosition::Polygon(
                        if j == 0 {
                            RingRole::Exterior
                        } else {
                            RingRole::Interior(j)
                        },
                        CoordinatePosition(0),
                    ),
                ));
            }

            for (i, point) in ring.0.iter().enumerate() {
                if utils::check_coord_is_not_finite(point) {
                    reason.push(ProblemAtPosition(
                        Problem::NotFinite,
                        ProblemPosition::Polygon(
                            if j == 0 {
                                RingRole::Exterior
                            } else {
                                RingRole::Interior(j)
                            },
                            CoordinatePosition(i),
                        ),
                    ));
                }
            }
        }

        let polygon_exterior = Polygon::new(self.exterior().clone(), vec![]);

        for (j, interior) in self.interiors().iter().enumerate() {
            if !polygon_exterior.contains(interior) {
                reason.push(ProblemAtPosition(
                    Problem::InteriorRingNotContainedInExteriorRing,
                    ProblemPosition::Polygon(RingRole::Interior(j), CoordinatePosition(0)),
                ));
            }

            let im = polygon_exterior.relate(interior);

            // Interior ring and exterior ring may only touch at point (not as a line)
            // and not cross
            match im.get(CoordPos::OnBoundary, CoordPos::Inside) {
                Dimensions::OneDimensional => {
                    reason.push(ProblemAtPosition(
                        Problem::IntersectingRingsOnALine,
                        ProblemPosition::Polygon(RingRole::Interior(j), CoordinatePosition(0)),
                    ));
                }
                _ => {}
            };
            let pol_interior1 = Polygon::new(interior.clone(), vec![]);
            for (i, interior2) in self.interiors().iter().enumerate() {
                if j != i {
                    let pol_interior2 = Polygon::new(interior2.clone(), vec![]);
                    let intersection_matrix = pol_interior1.relate(&pol_interior2);
                    match intersection_matrix.get(CoordPos::Inside, CoordPos::Inside) {
                        Dimensions::TwoDimensional => {
                            reason.push(ProblemAtPosition(
                                Problem::IntersectingRingsOnAnArea,
                                ProblemPosition::Polygon(
                                    RingRole::Interior(j),
                                    CoordinatePosition(0),
                                ),
                            ));
                        }
                        _ => {}
                    }
                    match intersection_matrix.get(CoordPos::OnBoundary, CoordPos::OnBoundary) {
                        Dimensions::OneDimensional => {
                            reason.push(ProblemAtPosition(
                                Problem::IntersectingRingsOnALine,
                                ProblemPosition::Polygon(
                                    RingRole::Interior(j),
                                    CoordinatePosition(0),
                                ),
                            ));
                        }
                        _ => {}
                    }
                }
            }
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

/// MultiPolygon is valid if:
/// - [x] all its polygons are valid,
/// - [x] elements do not overlaps (i.e. their interiors must not intersect)
/// - [x] elements touch only at points
impl Valid for MultiPolygon {
    fn is_valid(&self) -> bool {
        for (j, pol) in self.0.iter().enumerate() {
            if !pol.is_valid() {
                return false;
            }
            for (i, pol2) in self.0.iter().enumerate() {
                if j != i {
                    let im = pol.relate(pol2);
                    match im.get(CoordPos::Inside, CoordPos::Inside) {
                        Dimensions::TwoDimensional => {
                            return false;
                        }
                        _ => {}
                    }
                    match im.get(CoordPos::OnBoundary, CoordPos::OnBoundary) {
                        Dimensions::OneDimensional => {
                            return false;
                        }
                        _ => {}
                    }
                }
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        // Loop over all the polygons, collect the reasons of invalidity
        // and change the ProblemPosition to reflect the MultiPolygon
        let mut reason = Vec::new();

        for (j, polygon) in self.0.iter().enumerate() {
            let temp_reason = polygon.invalidity_reason();
            if let Some(temp_reason) = temp_reason {
                for ProblemAtPosition(problem, position) in temp_reason {
                    match position {
                        ProblemPosition::Polygon(ring_role, coord_pos) => {
                            reason.push(ProblemAtPosition(
                                problem,
                                ProblemPosition::MultiPolygon(
                                    GeometryPosition(j),
                                    ring_role,
                                    coord_pos,
                                ),
                            ));
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }
        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(reason)
        }
    }
}

impl Valid for GeometryCollection {
    fn is_valid(&self) -> bool {
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        None
    }
}
