use crate::{
    utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, RingRole, Valid,
};
use geo::coordinate_position::CoordPos;
use geo::dimensions::Dimensions;
use geo::{Contains, Relate};
use geo_types::Polygon;

/// In PostGIS, polygons must follow the following rules to be valid:
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
                        CoordinatePosition((ring.0.len() - 2) as isize),
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
                        CoordinatePosition(-1),
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
                            CoordinatePosition(i as isize),
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
                    ProblemPosition::Polygon(RingRole::Interior(j), CoordinatePosition(-1)),
                ));
            }

            let im = polygon_exterior.relate(interior);

            // Interior ring and exterior ring may only touch at point (not as a line)
            // and not cross
            match im.get(CoordPos::OnBoundary, CoordPos::Inside) {
                Dimensions::OneDimensional => {
                    reason.push(ProblemAtPosition(
                        Problem::IntersectingRingsOnALine,
                        ProblemPosition::Polygon(RingRole::Interior(j), CoordinatePosition(-1)),
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
                                    CoordinatePosition(-1),
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
                                    CoordinatePosition(-1),
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

#[cfg(test)]
mod tests {
    use crate::{
        CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition,
        RingRole, Valid,
    };
    use geo_types::{
        Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
        Point, Polygon,
    };

    #[test]
    fn test_polygon_valid() {
        // Unclosed rings are automatically closed by geo_types
        // so the following should be valid
        let p = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
                Coord { x: 0., y: 1. },
            ]),
            vec![],
        );
        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_polygon_valid_interior_ring_touches_exterior_ring() {
        // The following polygon contains an interior ring that touches
        // the exterior ring on one point.
        // This is valid according to the OGC spec.
        let p = Polygon::new(
            LineString::from(vec![(0., 0.), (4., 0.), (4., 4.), (0., 4.), (0., 0.)]),
            vec![LineString::from(vec![
                (0., 2.), // This point is on the exterior ring
                (2., 1.),
                (3., 2.),
                (2., 3.),
                (0., 2.),
            ])],
        );

        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_polygon_valid_interior_rings_touch_at_point() {
        // The following polygon contains two interior rings that touch
        // at one point.
        let p = Polygon::new(
            LineString::from(vec![(0., 0.), (4., 0.), (4., 4.), (0., 4.), (0., 0.)]),
            vec![
                LineString::from(vec![(1., 2.), (2., 1.), (3., 2.), (2., 3.), (1., 2.)]),
                LineString::from(vec![(3., 2.), (3.5, 1.), (3.75, 2.), (3.5, 3.), (3., 2.)]),
            ],
        );

        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_polygon_invalid_interior_rings_touch_at_line() {
        // The following polygon contains two interior rings that touch
        // on a line, this is not valid.
        let p = Polygon::new(
            LineString::from(vec![(0., 0.), (4., 0.), (4., 4.), (0., 4.), (0., 0.)]),
            vec![
                LineString::from(vec![(1., 2.), (2., 1.), (3., 2.), (2., 3.), (1., 2.)]),
                LineString::from(vec![
                    (3., 2.),
                    (2., 1.),
                    (3.5, 1.),
                    (3.75, 2.),
                    (3.5, 3.),
                    (3., 2.),
                ]),
            ],
        );

        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![
                ProblemAtPosition(
                    Problem::IntersectingRingsOnALine,
                    ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(-1))
                ),
                ProblemAtPosition(
                    Problem::IntersectingRingsOnALine,
                    ProblemPosition::Polygon(RingRole::Interior(1), CoordinatePosition(-1))
                )
            ])
        );
    }

    #[test]
    fn test_polygon_invalid_interior_rings_crosses() {
        // The following polygon contains two interior rings that cross
        // each other (they share some common area), this is not valid.
        let p = Polygon::new(
            LineString::from(vec![(0., 0.), (4., 0.), (4., 4.), (0., 4.), (0., 0.)]),
            vec![
                LineString::from(vec![(1., 2.), (2., 1.), (3., 2.), (2., 3.), (1., 2.)]),
                LineString::from(vec![
                    (2., 2.),
                    (2., 1.),
                    (3.5, 1.),
                    (3.75, 2.),
                    (3.5, 3.),
                    (3., 2.),
                ]),
            ],
        );

        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![
                ProblemAtPosition(
                    Problem::IntersectingRingsOnAnArea,
                    ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(-1))
                ),
                ProblemAtPosition(
                    Problem::IntersectingRingsOnAnArea,
                    ProblemPosition::Polygon(RingRole::Interior(1), CoordinatePosition(-1))
                )
            ])
        );
    }

    #[test]
    fn test_polygon_invalid_interior_ring_touches_exterior_ring_as_line() {
        // The following polygon contains an interior ring that touches
        // the exterior ring on one point.
        // This is valid according to the OGC spec.
        let p = Polygon::new(
            LineString::from(vec![(0., 0.), (4., 0.), (4., 4.), (0., 4.), (0., 0.)]),
            vec![LineString::from(vec![
                (0., 2.), // This point is on the exterior ring
                (0., 1.), // This point is on the exterior ring too
                (2., 1.),
                (3., 2.),
                (2., 3.),
                (0., 2.),
            ])],
        );

        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::IntersectingRingsOnALine,
                ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(-1))
            )])
        );
    }

    #[test]
    fn test_polygon_invalid_too_few_point_exterior_ring() {
        // Unclosed rings are automatically closed by geo_types
        // but there is still two few points in this ring
        // to be a non-empty polygon
        let p = Polygon::new(
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            vec![],
        );
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::Polygon(RingRole::Exterior, CoordinatePosition(1))
            )])
        );
    }

    #[test]
    fn test_polygon_invalid_spike() {
        // The following polygon contains a spike
        let p = Polygon::new(
            LineString::from(vec![
                (0., 0.),
                (4., 0.),
                (4., 4.),
                (2., 4.),
                (2., 6.),
                (2., 4.),
                (0., 4.),
                (0., 0.),
            ]),
            vec![],
        );

        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::SelfIntersection,
                ProblemPosition::Polygon(RingRole::Exterior, CoordinatePosition(-1))
            )])
        );
    }

    #[test]
    fn test_polygon_invalid_exterior_is_not_simple() {
        // The exterior ring of this polygon is not simple (i.e. it has a self-intersection)
        let p = Polygon::new(
            LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 4., y: 0. },
                Coord { x: 0., y: 2. },
                Coord { x: 4., y: 2. },
                Coord { x: 0., y: 0. },
            ]),
            vec![],
        );
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::SelfIntersection,
                ProblemPosition::Polygon(RingRole::Exterior, CoordinatePosition(-1))
            )])
        );
    }

    #[test]
    fn test_polygon_invalid_interior_not_fully_contained_in_exterior() {
        let p = Polygon::new(
            LineString::from(vec![
                (0.5, 0.5),
                (3., 0.5),
                (3., 2.5),
                (0.5, 2.5),
                (0.5, 0.5),
            ]),
            vec![LineString::from(vec![
                (1., 1.),
                (1., 2.),
                (2.5, 2.),
                (3.5, 1.),
                (1., 1.),
            ])],
        );
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::InteriorRingNotContainedInExteriorRing,
                ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(-1))
            )])
        );
    }
}
