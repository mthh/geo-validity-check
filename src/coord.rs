use crate::{utils, Problem, ProblemAtPosition, ProblemPosition, ProblemReport, Valid};
use geo::GeoFloat;
use geo_types::Coord;

impl<T> Valid for Coord<T>
where
    T: GeoFloat,
{
    fn is_valid(&self) -> bool {
        if utils::check_coord_is_not_finite(self) {
            return false;
        }
        true
    }
    fn explain_invalidity(&self) -> Option<ProblemReport> {
        let mut reason = Vec::new();

        if utils::check_coord_is_not_finite(self) {
            reason.push(ProblemAtPosition(
                Problem::NotFinite,
                ProblemPosition::Point,
            ));
        }

        // Return the reason(s) of invalidity, or None if valid
        if reason.is_empty() {
            None
        } else {
            Some(ProblemReport(reason))
        }
    }
}
