use crate::{
    CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, RingRole,
    Valid,
};
use geo::coordinate_position::CoordPos;
use geo::dimensions::Dimensions;
use geo::Relate;
use geo_types::MultiPolygon;

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
                    if pol == pol2 {
                        return false;
                    }
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
    fn explain_invalidity(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        // Loop over all the polygons, collect the reasons of invalidity
        // and change the ProblemPosition to reflect the MultiPolygon
        for (j, polygon) in self.0.iter().enumerate() {
            let temp_reason = polygon.explain_invalidity();
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

            // Special case for MultiPolygon: elements must not overlap and must touch only at points
            for (i, pol2) in self.0.iter().enumerate() {
                if j != i {
                    if polygon == pol2 {
                        reason.push(ProblemAtPosition(
                            Problem::ElementsAreIdentical,
                            ProblemPosition::MultiPolygon(
                                GeometryPosition(j),
                                RingRole::Exterior,
                                CoordinatePosition(-1),
                            ),
                        ));
                    } else {
                        let im = polygon.relate(pol2);
                        match im.get(CoordPos::Inside, CoordPos::Inside) {
                            Dimensions::TwoDimensional => {
                                reason.push(ProblemAtPosition(
                                    Problem::ElementsOverlaps,
                                    ProblemPosition::MultiPolygon(
                                        GeometryPosition(j),
                                        RingRole::Exterior,
                                        CoordinatePosition(-1),
                                    ),
                                ));
                            }
                            _ => {}
                        }
                        match im.get(CoordPos::OnBoundary, CoordPos::OnBoundary) {
                            Dimensions::OneDimensional => {
                                reason.push(ProblemAtPosition(
                                    Problem::ElementsTouchOnALine,
                                    ProblemPosition::MultiPolygon(
                                        GeometryPosition(j),
                                        RingRole::Exterior,
                                        CoordinatePosition(-1),
                                    ),
                                ));
                            }
                            _ => {}
                        }
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
    use geo_types::{LineString, MultiPolygon, Polygon};

    #[test]
    fn test_multipolygon_invalid() {
        // The following multipolygon contains two invalid polygon
        // and it is invalid itself because the two polygons of the multipolygon are not disjoint
        // (here they are identical)
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
            mp.explain_invalidity(),
            Some(vec![
                ProblemAtPosition(
                    Problem::InteriorRingNotContainedInExteriorRing,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(0),
                        RingRole::Interior(0),
                        CoordinatePosition(-1)
                    )
                ),
                ProblemAtPosition(
                    Problem::ElementsAreIdentical,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(0),
                        RingRole::Exterior,
                        CoordinatePosition(-1)
                    )
                ),
                ProblemAtPosition(
                    Problem::InteriorRingNotContainedInExteriorRing,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(1),
                        RingRole::Interior(0),
                        CoordinatePosition(-1)
                    )
                ),
                ProblemAtPosition(
                    Problem::ElementsAreIdentical,
                    ProblemPosition::MultiPolygon(
                        GeometryPosition(1),
                        RingRole::Exterior,
                        CoordinatePosition(-1)
                    )
                )
            ])
        );
    }
}
