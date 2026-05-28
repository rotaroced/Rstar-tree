mod rect;
mod rtree;
pub mod to_svg;
pub use rtree::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        for _ in 0..1 {
            let mut t = RTree::<i32, 5, 2>::default();

            let p = [0., 1e300];
            let mut min_d = f64::INFINITY;
            let mut best = 0;

            for _ in 0..10_000_000 {
                let x = (rand::random::<f64>()) as f64;
                let y = (rand::random::<f64>()) as f64;

                let value = rand::random::<i32>();
                t.insert(&[x, y], value);
                let d = ((x - p[0]) * (x - p[0]) + (y - p[1]) * (y - p[1])).sqrt();
                if d < min_d {
                    min_d = d;
                    best = value;
                }
            }
            println!("\n\n");
            let t0 = std::time::Instant::now();
            let (x, dd, best_rtree) = t.closest(&p);
            let t1 = std::time::Instant::now();
            let delta = t1 - t0;
            println!("time for search: {delta:?} ({t0:?} - {t1:?})");
            println!("actual: {min_d} {best:?}");
            println!("found:  {} {}", dd.sqrt(), best_rtree);
            println!("{x:?}");
            //println!("{:?}", t);
            // svg::save(
            //     "big.svg",
            //     &to_svg::bidimensional_rtree_to_svg(t)
            //         .add(to_svg::point_to_svg(&p).set("fill", "red")),
            // )
            // .unwrap();
            assert!((dd.sqrt() - min_d) < 1e-10);
            // assert!(false);
        }
        panic!()
    }
}
