use crate::{utils, GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
use geo_types::MultiPoint;

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

#[cfg(test)]
mod tests {
    use crate::{GeometryPosition, Problem, ProblemAtPosition, ProblemPosition, Valid};
    use geo_types::{MultiPoint, Point};

    #[test]
    fn test_multipoint_valid() {
        let mp = MultiPoint(vec![Point::new(0., 0.), Point::new(1., 1.)]);
        assert!(mp.is_valid());
        assert!(mp.invalidity_reason().is_none());
    }

    #[test]
    fn test_multipoint_invalid() {
        let mp = MultiPoint(vec![
            Point::new(0., f64::INFINITY),
            Point::new(f64::NAN, 1.),
        ]);
        assert!(!mp.is_valid());
        assert_eq!(
            mp.invalidity_reason(),
            Some(vec![
                ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(0))
                ),
                ProblemAtPosition(
                    Problem::NotFinite,
                    ProblemPosition::MultiPoint(GeometryPosition(1))
                )
            ])
        );
    }
}
