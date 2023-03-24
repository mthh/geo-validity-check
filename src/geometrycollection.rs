use crate::{GeometryPosition, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::GeometryCollection;

/// GeometryCollection is valid if all its elements are valid
impl Valid for GeometryCollection {
    fn is_valid(&self) -> bool {
        for geometry in self.0.iter() {
            if !geometry.is_valid() {
                return false;
            }
        }
        true
    }
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>> {
        let mut reason = Vec::new();

        // Loop over all the geometries, collect the reasons of invalidity
        // and change the ProblemPosition to reflect the GeometryCollection
        for (i, geometry) in self.0.iter().enumerate() {
            let temp_reason = geometry.invalidity_reason();
            if let Some(temp_reason) = temp_reason {
                for ProblemAtPosition(problem, position) in temp_reason {
                    reason.push(ProblemAtPosition(
                        problem,
                        ProblemPosition::GeometryCollection(
                            GeometryPosition(i),
                            Box::new(position),
                        ),
                    ));
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
        CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, Valid,
    };
    use geo_types::{Coord, Geometry, GeometryCollection, LineString, Point};

    #[test]
    fn test_geometrycollection_contain_invalid_element() {
        let gc = GeometryCollection(vec![
            Geometry::Point(Point::new(0., 0.)),
            Geometry::LineString(LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 1., y: 1. },
            ])),
            Geometry::LineString(LineString(vec![
                Coord { x: 0., y: 0. },
                Coord { x: 0., y: 0. },
            ])),
        ]);
        assert!(!gc.is_valid());
        assert_eq!(
            gc.invalidity_reason(),
            Some(vec![ProblemAtPosition(
                Problem::TooFewPoints,
                ProblemPosition::GeometryCollection(
                    GeometryPosition(2),
                    Box::new(ProblemPosition::LineString(CoordinatePosition(0)))
                )
            )])
        );
    }
}
