use geo_validity_check::{Valid, ProblemReport};
use geo_types::{Coord, Point, LineString, MultiLineString, Polygon, MultiPolygon, GeometryCollection};

fn main() {
    let point = Point::new(0., f64::NAN);

    println!("Point is valid: {}", point.is_valid());
    println!("Point is invalid because: {}", ProblemReport(point.explain_invalidity().unwrap()));

    let line = LineString(vec![
        Coord { x: 0., y: 0. },
        Coord { x: 0., y: 0. },
    ]);

    println!("Line is valid: {}", line.is_valid());
    println!("Line is invalid because: {}", ProblemReport(line.explain_invalidity().unwrap()));

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

    println!("Polygon is valid: {}", p.is_valid());
    println!("Polygon is invalid because: {}", ProblemReport(p.explain_invalidity().unwrap()));

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

    println!("MultiPolygon is valid: {}", mp.is_valid());
    println!(
        "MultiPolygon is invalid because: {}",
        ProblemReport(mp.explain_invalidity().unwrap())
    );
}