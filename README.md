# geo-validity-check

Expose a `Valid` trait to check if rust [geo-types](https://crates.io/crates/geo-types) geometries are valid.

`Valid` trait has the following signature:

```rust
trait Valid {
    fn is_valid(&self) -> bool;
    fn invalidity_reason(&self) -> Option<Vec<ProblemAtPosition>>;
}
```

The result of the invalidity reason can be formatted as a string as it implements the `Display` trait.

## Checks implemented

- [x] `Coord` and `Point` use finite numbers
- [x] `MultiPoint` is made of valid points
- [x] `Rect`, `Line` and `Triangle` are made of valid coords
- [x] `Triangle` are not empty or degenerate (i.e. all points are different and not collinear)
- [ ] `Line` is not of 0-length (i.e. both points are not the same)
- [x] `LineString` is made of valid points
- [x] `LineString` is not empty
- [x] `LineString` has at least two different points
- [x] `MultiLineString` is made of valid linestrings
- [x] `Polygon` rings have at least 4 points (including the closing point)
- [x] `Polygon` interior rings are contained in the exterior ring (but can touch it on a point)
- [x] `Polygon` interior rings don't cross each other (but can touch on a point)
- [x] `MultiPolygon` components don't cross each other (but can touch on a point)
- [x] `MultiPolygon` is made of valid polygons
- [ ] `GeometryCollection` is made of valid geometries

## Example

```rust

These tests are mostly based on the [OGC Simple Features for SQL specification](https://www.ogc.org/standards/sfa)
but also include some additional checks.

```rust
use geo_validity_check::Valid;

let line1 = LineString::from(vec![(0., 0.), (1., 1.)]);
let line2 = LineString::from(vec![(0., 0.), (0., 0.)]);
let line3 = LineString::from(vec![(0., 0.), (f64::NAN, f64::NAN), (1., 1.)]);

assert!(line1.is_valid());
assert!(!line2.is_valid());
assert!(!line3.is_valid());

println!("{}", line2.invalidity_reason().unwrap()); // "LineString is empty."
println!("{}", line3.invalidity_reason().unwrap()); // "Coordinates (of point 1) have to be finite numbers."

let polygon = Polygon::new(
    LineString::from(vec![(0.5, 0.5), (3., 0.5), (3., 2.5), (0.5, 2.5), (0.5, 0.5)]),
    vec![LineString::from(vec![(1., 1.), (1., 2.), (2.5, 2.), (3.5, 1.), (1., 1.)])],
);

assert!(!polygon.is_valid());
println!("{}", polygon.invalidity_reason().unwrap()); // "Interior ring 0 intersects the exterior ring."

let multipolygon = MultiPolygon(vec![
    Polygon::new(
        LineString::from(vec![(0.5, 0.5), (3., 0.5), (3., 2.5), (0.5, 2.5), (0.5, 0.5)]),
        vec![LineString::from(vec![(1., 1.), (1., 2.), (2.5, 2.), (3.5, 1.), (1., 1.)])],
    ),
    Polygon::new(
        LineString::from(vec![(0.5, 0.5), (3., 0.5), (3., 2.5), (0.5, 2.5), (0.5, 0.5)]),
        vec![LineString::from(vec![(1., 1.), (1., 2.), (2.5, 2.), (3.5, 1.), (1., 1.)])],
    ),
]);

assert!(!multipolygon.is_valid());
println!("{}", multipolygon.invalidity_reason().unwrap());
// "Inner ring 0 intersects the exterior ring (Polygon 0).
// Inner ring 0 intersects the exterior ring (Polygon 1)."
```