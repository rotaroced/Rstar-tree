use core::cmp::{max, min};
use core::ops::Mul;
use num_traits::real::Real;
pub use ordered_float::NotNan;
use std::ops::{Add, Not};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Rect(pub Vec<(NotNan<f64>, NotNan<f64>)>);

impl Rect {
    pub fn empty_rect(dim: usize) -> Self {
        Self(
            (0..dim)
                .map(|_| (NotNan::default(), NotNan::default()))
                .collect(),
        )
    }
    #[inline]
    pub fn dim(&self) -> usize {
        self.0.len()
    }

    pub fn merge(r1: &Rect, r2: &Rect) -> Rect {
        debug_assert!(r1.eq_dim(r2));
        let mut r = Self::empty_rect(r1.dim());
        for i in 0..r1.dim() {
            r.0[i] = (min(r1.0[i].0, r2.0[i].0), max(r1.0[i].1, r2.0[i].1));
        }
        r
    }

    pub fn from_point(p: &[f64]) -> Self {
        let mut r = Self::empty_rect(p.len());
        // println!("{p:?}");
        for (i, &x) in p.iter().enumerate() {
            let x = x.try_into().unwrap();
            r.0[i] = (x, x)
        }
        r
    }

    pub fn contains(&self, p: &[f64]) -> bool {
        debug_assert_eq!(self.dim(), p.len());
        self.0
            .iter()
            .zip(p)
            .all(|((lower, upper), x)| &lower.into_inner() <= x && x <= &upper.into_inner())
    }

    pub fn volume(&self) -> NotNan<f64> {
        self.0
            .iter()
            .map(|(l, u)| u - l)
            .fold(NotNan::try_from(1.).unwrap(), |a, b| a * b)
    }

    /// Returns the intersection of two rectangles.
    /// if the intersection is empty, returns rectangle of volume 0 (which therefore isn't the
    /// actual intersection of the rectangles).
    pub fn inter(&self, r: &Rect) -> Rect {
        debug_assert!(self.eq_dim(r));
        Rect(
            self.0
                .iter()
                .zip(r.0.iter())
                .map(|(&x, &y)| segments_intersection(x, y))
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
        )
    }

    // a quantity proportionnal to the sum of the lengths of all edges.
    pub fn margin(&self) -> NotNan<f64> {
        self.0
            .iter()
            .map(|&(x, y)| <NotNan<f64> as Real>::sqrt(x * x + y * y))
            .fold(NotNan::try_from(0.).unwrap(), |x, y| x + y)
    }

    pub fn dist_from_point(&self, p: &[f64]) -> f64 {
        debug_assert_eq!(self.dim(), p.len());
        self.0
            .iter()
            .zip(p.into_iter())
            .map(segment_dist_from_point)
            .fold(0., |a, x| a + x * x)
    }

    fn eq_dim(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
    }
}

#[inline]
fn segments_intersection(
    a: (NotNan<f64>, NotNan<f64>),
    b: (NotNan<f64>, NotNan<f64>),
) -> (NotNan<f64>, NotNan<f64>) {
    let x = max(a.0, b.0);
    let y = min(a.1, b.1);
    if x > y { (x, x) } else { (x, y) }
}

#[inline]
fn segment_dist_from_point((s, x): (&(NotNan<f64>, NotNan<f64>), &f64)) -> f64 {
    if x >= &s.0.into_inner() && x <= &s.1.into_inner() {
        0.
    } else if x > &s.0.into_inner() {
        x - s.1.into_inner()
    } else {
        s.0.into_inner() - x
    }
}

#[cfg(test)]
mod tests {
    use num_traits::{Float, FromPrimitive};
    use ordered_float::NotNan;

    use crate::rect::Rect;

    #[test]
    fn dist_from_point() {
        let p = [5., 3.];

        let r1 = Rect(vec![
            (NotNan::from_f64(0.).unwrap(), NotNan::from_f64(3.).unwrap()),
            (NotNan::from_f64(1.).unwrap(), NotNan::from_f64(2.).unwrap()),
        ]);

        println!("{}", r1.dist_from_point(&p));
        assert!((r1.dist_from_point(&p) - 5.) < 1e-15);
    }
}
