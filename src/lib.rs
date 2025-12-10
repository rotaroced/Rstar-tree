mod rect;
mod rtree;
pub mod to_svg;
pub use rtree::*;

#[cfg(test)]
mod tests {
    use core::f64;

    use num_traits::{Float, real::Real};
    use svg::{Document, node::element::Rectangle};

    use super::*;

    #[test]
    fn it_works() {
        for _ in 0..1 {
            let mut t = RTree::<i32, 16>::new(2);

            let p = [372., 142.];
            let mut min_d = f64::INFINITY;
            let mut best = 0;

            for i in 0..30000 {
                let x = (rand::random::<u64>() % 2000) as f64;
                let y = (rand::random::<u64>() % 1000) as f64;
                let value = rand::random::<i32>();
                t.insert(&[x, y], value);
                let d = ((x - p[0]) * (x - p[0]) + (y - p[1]) * (y - p[1])).sqrt();
                if d < min_d {
                    min_d = d;
                    best = value;
                }
            }
            println!("\n\n");
            let (_, dd, best_rtree) = t.closest(&p);
            println!("actual: {min_d} {best:?}");
            println!("found:  {} {}", dd.sqrt(), best_rtree);
            println!("{:?}", t);
            svg::save(
                "big.svg",
                &to_svg::bidimensional_rtree_to_svg(t)
                    .add(to_svg::point_to_svg(&p).set("fill", "red")),
            )
            .unwrap();
            assert!((dd.sqrt() - min_d) < 1e-10);
            // assert_eq!(best, *best_rtree);
        }
    }
}
