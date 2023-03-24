use crate::{utils, CoordinatePosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::LineString;

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
                    ProblemPosition::LineString(CoordinatePosition(i as isize)),
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

#[cfg(test)]
mod tests {
    use crate::{
        CoordinatePosition, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, Valid,
    };
    use geo_types::{Coord, LineString, Point};

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
}
