use geo_types::{
    Coord, GeometryCollection, LineString, MultiLineString, MultiPolygon, Point, Polygon, Triangle,
};
use geo_validity_check::{ProblemReport, Valid};

fn main() {
    let point = Point::new(0., f64::NAN);

    println!("=================================================");
    println!("Point is valid: {}", point.is_valid());
    println!(
        "Point is invalid because: {}",
        point.explain_invalidity().unwrap()
    );

    let line = LineString(vec![Coord { x: 0., y: 0. }, Coord { x: 0., y: 0. }]);

    println!("=================================================");
    println!("LineString is valid: {}", line.is_valid());
    println!(
        "LineString is invalid because: {}",
        line.explain_invalidity().unwrap()
    );

    let ml = MultiLineString(vec![
        LineString::from(vec![(0., 0.), (1., 1.)]),
        LineString::from(vec![(0., 0.), (0., 0.)]),
    ]);

    println!("=================================================");
    println!("MultiLineString is valid: {}", ml.is_valid());
    println!(
        "MultiLineString is invalid because: {}",
        ml.explain_invalidity().unwrap()
    );

    let t = Triangle::new((0., 0.).into(), (0., 0.).into(), (4., 4.).into());

    println!("=================================================");
    println!("Triangle is valid: {}", t.is_valid());
    println!(
        "Triangle is invalid because: {}",
        t.explain_invalidity().unwrap()
    );

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

    println!("=================================================");
    println!("Polygon is valid: {}", p.is_valid());
    println!(
        "Polygon is invalid because: {}",
        p.explain_invalidity().unwrap()
    );

    let p = Polygon::new(LineString::from(vec![(0., 0.), (4., 0.)]), vec![]);

    println!("=================================================");
    println!("Polygon is valid: {}", p.is_valid());
    println!(
        "Polygon is invalid because: {}",
        p.explain_invalidity().unwrap()
    );

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

    println!("=================================================");
    println!("MultiPolygon is valid: {}", mp.is_valid());
    println!(
        "MultiPolygon is invalid because: {}",
        mp.explain_invalidity().unwrap()
    );

    let gc = GeometryCollection(vec![
        point.into(),
        line.into(),
        ml.into(),
        p.into(),
        mp.into(),
    ]);

    println!("=================================================");
    println!("GeometryCollection is valid: {}", gc.is_valid());
    println!(
        "GeometryCollection is invalid because: {}",
        gc.explain_invalidity().unwrap()
    );
}
