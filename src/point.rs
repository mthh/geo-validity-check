use crate::{ProblemAtPosition, Valid};
use geo_types::Point;

/// In PostGIS, Point don't have any validity constraint.
/// Here we choose to check that points are finite numbers (i.e. not NaN or infinite)
impl Valid for Point {
    fn is_valid(&self) -> bool {
        self.0.is_valid()
    }
    fn explain_invalidity(&self) -> Option<Vec<ProblemAtPosition>> {
        self.0.explain_invalidity()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Problem, ProblemAtPosition, ProblemPosition, Valid};
    use geo_types::Point;

    #[test]
    fn test_point_valid() {
        let p = Point::new(0., 0.);
        assert!(p.is_valid());
        assert!(p.explain_invalidity().is_none());
    }

    #[test]
    fn test_point_invalid() {
        let p = Point::new(f64::NAN, f64::NAN);
        assert!(!p.is_valid());
        assert_eq!(
            p.explain_invalidity(),
            Some(vec![ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point
            )])
        );
    }
}
