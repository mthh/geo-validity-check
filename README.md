# geo-validity-check

> [!WARNING]
> Note that validation has been added directly to [`geo`](https://github.com/georust/geo) in december 2024 (see [#1279](https://github.com/georust/geo/pull/1279))
> on the basis of this crate, plus improvements by the `geo` maintainers.  
> You can now find a `Validation` trait in the `geo` crate that provides the same functionality as this crate.  
> I therefore do not plan to maintain this crate anymore and the repository will be archived soon.


Expose a `Valid` trait to check if rust [geo-types](https://crates.io/crates/geo-types) geometries are valid.

`Valid` trait has the following signature:

```rust
trait Valid {
    fn is_valid(&self) -> bool;
    fn explain_invalidity(&self) -> Option<ProblemReport>;
}
```

The result of the invalidity reason is provided in a `ProblemReport` struct (it contains a `Vec` of `(Problem, ProblemPosition)`,
two enums that respectively represent the type of problem and the position of the problem in the tested geometry - having
this machine-readable information could be useful to try to fix the geometry).
This `ProblemReport` result can also be formatted as a string as it implements the `Display` trait.

## Checks implemented

- [x] `Coord` and `Point` use finite numbers
- [x] `MultiPoint` is made of valid points
- [x] `Rect`, `Line` and `Triangle` are made of valid coords
- [x] `Triangle` are not empty or degenerate (i.e. all points are different and not collinear)
- [x] `Line` is not of 0-length (i.e. both points are not the same)
- [x] `LineString` is made of valid points
- [x] `LineString` is not empty
- [x] `LineString` has at least two different points
- [x] `MultiLineString` is made of valid linestrings
- [x] `Polygon` rings are made of valid points
- [x] `Polygon` rings have at least 4 points (including the closing point)
- [x] `Polygon` interior rings are contained in the exterior ring (but can touch it on a point)
- [x] `Polygon` interior rings don't cross each other (but can touch on a point)
- [x] `MultiPolygon` components don't cross each other (but can touch on a point)
- [x] `MultiPolygon` is made of valid polygons
- [x] `GeometryCollection` is made of valid geometries

Verification is done against GEOS
(any geometry invalid according to GEOS should be invalid according to this crate - the inverse doesn't have to be true since we are doing some extra check).

## Example

```rust
use geo_validity_check::Valid;
use geo_types::{Point, LineString, Polygon, MultiPolygon};

let line1 = LineString::from(vec![(0., 0.), (1., 1.)]);
let line2 = LineString::from(vec![(0., 0.), (0., 0.)]);
let line3 = LineString::from(vec![(0., 0.), (f64::NAN, f64::NAN), (1., 1.)]);

assert!(line1.is_valid());
assert!(!line2.is_valid());
assert!(!line3.is_valid());

println!("{}", line2.invalidity_reason().unwrap()); // "LineString has too few points at coordinate 0 of the LineString"
println!("{}", line3.invalidity_reason().unwrap()); // "Coordinate is not finite (NaN or infinite) at coordinate 1 of the LineString"

let polygon = Polygon::new(
    LineString::from(vec![(0.5, 0.5), (3., 0.5), (3., 2.5), (0.5, 2.5), (0.5, 0.5)]),
    vec![LineString::from(vec![(1., 1.), (1., 2.), (2.5, 2.), (3.5, 1.), (1., 1.)])],
);

assert!(!polygon.is_valid());
println!("{}", polygon.invalidity_reason().unwrap()); // "The interior ring of a Polygon is not contained in the exterior ring on the interior ring n°0"

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
// "The interior ring of a Polygon is not contained in the exterior ring on the interior ring n°0 of the Polygon n°0 of the MultiPolygon
// Two Polygons of MultiPolygons are identical on the exterior ring of the Polygon n°0 of the MultiPolygon
// The interior ring of a Polygon is not contained in the exterior ring on the interior ring n°0 of the Polygon n°1 of the MultiPolygon
// Two Polygons of MultiPolygons are identical on the exterior ring of the Polygon n°1 of the MultiPolygon"

```

## TODO / Ideas

- [ ] Improve the description of the invalidity reason (e.g. *"Interior ring 0 intersects the exterior ring"* could be *"Interior ring 0 intersects the exterior ring at point (1.5, 1.5)"*)

- [ ] Add a `make_valid` or `fix_invalidity` method to try to fix the geometry (e.g. by removing the invalid points ?)

- [ ] Return the first invalidity reason found (instead of all of them) in `invalidity_reason` method ? (because some other checks could fail because of the first invalidity reason)

- [ ] Implement a rule that states that a `Polygon` is valid if the polygon interior is simply connected (i.e. the rings must not touch in a way that splits the polygon into more than one part) ?

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
