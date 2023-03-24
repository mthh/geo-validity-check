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
    fn test_point_valid() {
        let p = Point::new(0., 0.);
        assert!(p.is_valid());
        assert!(p.invalidity_reason().is_none());
    }

    #[test]
    fn test_point_invalid() {
        let p = Point::new(f64::NAN, f64::NAN);
        assert!(!p.is_valid());
        assert_eq!(
            p.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point
            )])
        );
    }

    #[test]
    fn test_multipoint() {
        let mp = MultiPoint(vec![Point::new(0., 0.), Point::new(1., 1.)]);
        assert!(mp.is_valid());
        assert!(mp.invalidity_reason().is_none());
    }

    #[test]
    fn test_linestring_valid() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]);
        assert!(ls.is_valid());
        assert!(ls.invalidity_reason().is_none());
    }

    #[test]
    fn test_linestring_invalid_too_few_points_without_duplicate() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }]);
        assert!(!ls.is_valid());
        assert_eq!(
            ls.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::LineString(CoordinatePosition(0))
            )])
        );
    }

    #[test]
    fn test_linestring_invalid_too_few_points_with_duplicate() {
        let ls = LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 0., y: 0. }]);
        assert!(!ls.is_valid());
        assert_eq!(
            ls.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::LineString(CoordinatePosition(0))
            )])
        );
    }

    #[test]
    fn test_multilinestring_valid() {
        let mls = MultiLineString(vec![
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            LineString(vec![Coord { x: 3., y: 1. }, Coord { x: 4., y: 1. }]),
        ]);
        assert!(mls.is_valid());
        assert!(mls.invalidity_reason().is_none());
    }

    #[test]
    fn test_multilinestring_invalid_too_few_points_with_duplicate() {
        // The second LineString (at position 1) of this MultiLineString
        // is not valid because it has only one (deduplicated) point
        let mls = MultiLineString(vec![
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 1., y: 1. }]),
            LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 0., y: 0. }]),
        ]);
        assert!(!mls.is_valid());
        assert_eq!(
            mls.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::MultiLineString(GeometryPosition(1), CoordinatePosition(0))
            )])
        );
    }

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
                    ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(0))
                ),
                ProblemAtPosition(
                    Problem::IntersectingRingsOnALine,
                    ProblemPosition::Polygon(RingRole::Interior(1), CoordinatePosition(0))
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
                    ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(0))
                ),
                ProblemAtPosition(
                    Problem::IntersectingRingsOnAnArea,
                    ProblemPosition::Polygon(RingRole::Interior(1), CoordinatePosition(0))
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
                ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(0))
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
                ProblemPosition::Polygon(RingRole::Exterior, CoordinatePosition(0))
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
                ProblemPosition::Polygon(RingRole::Exterior, CoordinatePosition(0))
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
                ProblemPosition::Polygon(RingRole::Interior(0), CoordinatePosition(0))
            )])
        );
    }

    #[test]
    fn test_multipolygon_valid() {
        //
        let mp = MultiPolygon(vec![
            Polygon::new(
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
            ),
            Polygon::new(
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
            ),
        ]);
        assert!(!mp.is_valid());
        assert_eq!(
            mp.invalidity_reason(),
            Some(vec![
                ProblemAtPosition(
                    Problem::InteriorRingNotContainedInExteriorRing,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(0),
                        RingRole::Interior(0),
                        CoordinatePosition(0)
                    )
                ),
                ProblemAtPosition(
                    Problem::InteriorRingNotContainedInExteriorRing,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(1),
                        RingRole::Interior(0),
                        CoordinatePosition(0)
                    )
                )
            ])
        );
    }

    #[test]
    fn test_geometrycollection() {
        let gc = GeometryCollection(vec![Geometry::Point(Point::new(0., 0.))]);
        assert!(gc.is_valid());
        assert!(gc.invalidity_reason().is_none());
    }
}
