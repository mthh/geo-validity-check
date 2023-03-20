# geo-validity-check

Expose a `Valid` trait to check if georust/geo geometries are valid.

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
println!("{}", polygon.invalidity_reason().unwrap()); // "Inner ring 0 intersects the exterior ring."
```