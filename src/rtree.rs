use super::rect::*;
use core::f64;
use std::collections::BinaryHeap;
use std::rc::Rc;
use std::{fmt, usize};

/// M is the maximum number of children per node.
/// algorithms from this paper : https://dl.acm.org/doi/pdf/10.1145/93605.98741
#[derive(Clone, Debug)]
pub enum RTree<T: fmt::Debug + Clone, const M: usize> {
    InternalNode(Rect, Vec<Rc<RTree<T, M>>>),
    LeafNode(Rect, Vec<Rc<RTree<T, M>>>),
    Leaf(Vec<f64>, Rc<T>),
}

use RTree::*;

impl<T: fmt::Debug + Clone, const M: usize> RTree<T, M> {
    #[inline]
    pub fn new(dim: usize) -> RTree<T, M> {
        LeafNode(Rect::empty_rect(dim), vec![])
    }

    #[inline]
    /// TODO: find a way to remove the two `clone` there, this function is used everywhere
    pub fn get_rect(&self) -> Rect {
        match self {
            Self::InternalNode(r, _) => r.clone(),
            Self::LeafNode(r, _) => r.clone(),
            Self::Leaf(p, _) => Rect::from_point(p),
        }
    }

    /// returns a tuple containing :
    /// - the closest point of the R-tree
    /// - the distance to this point
    /// - the value of this point
    pub fn closest(&self, p: &[f64]) -> (Vec<f64>, f64, Rc<T>) {
        let mut pq = BinaryHeap::new();

        match self {
            Leaf(x, e) => unreachable!(),
            LeafNode(r, v) => {
                for t in v {
                    pq.push(WeightedTree {
                        key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(t),
                    });
                }
            }
            InternalNode(r, v) => {
                for t in v {
                    pq.push(WeightedTree {
                        key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(t),
                    });
                }
            }
        }

        loop {
            let WeightedTree { key, value } = pq.pop().unwrap();

            match value.as_ref() {
                Leaf(x, e) => {
                    return (x.clone(), points_distance(x, p), Rc::clone(e));
                }
                LeafNode(r, v) => {
                    for t in v {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        })
                    }
                }
                InternalNode(r, v) => {
                    for t in v {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        })
                    }
                }
            }
        }
    }
    pub fn k_closest(&self, p: &[f64], mut k: usize) -> Vec<(Vec<f64>, NotNan<f64>, Rc<T>)> {
        let mut pq = BinaryHeap::new();

        match self {
            Leaf(x, e) => unreachable!(),
            LeafNode(r, v) => {
                for t in v {
                    pq.push(WeightedTree {
                        key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(t),
                    });
                }
            }
            InternalNode(r, v) => {
                for t in v {
                    pq.push(WeightedTree {
                        key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(t),
                    });
                }
            }
        }

        let mut res = vec![];

        while k > 0 && !pq.is_empty() {
            let WeightedTree { key, value } = pq.pop().unwrap();

            match value.as_ref() {
                Leaf(x, e) => {
                    res.push((
                        x.clone(),
                        points_distance(x, p).try_into().unwrap(),
                        Rc::clone(e),
                    ));
                    k -= 1
                }
                LeafNode(r, v) => {
                    for t in v {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        })
                    }
                }
                InternalNode(r, v) => {
                    for t in v {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        })
                    }
                }
            }
        }

        res
    }

    /// inserts a node and returns Some((l, r)) when the node is split.
    pub(crate) fn _insert(&mut self, p: &[f64], value: T) -> Option<(Self, Self)> {
        match self {
            Leaf(_, _) => panic!("tried illegal insertion in a leaf"),
            LeafNode(r, v) => {
                v.push(Rc::new(Leaf(Vec::from(p), Rc::new(value))));
                if v.len() == 1 {
                    *r = Rect::from_point(&p);
                    return None;
                }
                *r = Rect::merge(r, &Rect::from_point(p));
                if v.len() > M {
                    Some(self.split())
                } else {
                    None
                }
            }
            InternalNode(bb, v) => {
                *bb = Rect::merge(bb, &Rect::from_point(p));

                let mut min_i = 0;
                let mut min_enlargement = f64::INFINITY;
                let mut min_rect_volume = 0.;

                for (i, child) in v.iter().enumerate() {
                    let r = child.get_rect();
                    let enlargement = Rect::merge(&r, &Rect::from_point(p)).volume().into_inner()
                        - r.volume().into_inner();
                    // resolves ties using volume
                    // println!("{child:?}, {enlargement} {:?}\n", r.volume());
                    if (enlargement, r.volume().into_inner()) <= (min_enlargement, min_rect_volume)
                    {
                        min_i = i;
                        min_enlargement = enlargement;
                        min_rect_volume = r.volume().into_inner();
                    }
                }
                // println!("chose {:?}", v[min_i]);

                let res = Self::_insert(Rc::make_mut(&mut v[min_i]), p, value);
                if let Some((l, r)) = res {
                    v[min_i] = Rc::new(l);
                    v.push(Rc::new(r));
                }

                if v.len() > M {
                    Some(self.split())
                } else {
                    None
                }
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, p: &[f64], value: T) {
        if let Some((l, r)) = self._insert(p, value) {
            *self = Self::InternalNode(
                Rect::merge(&l.get_rect(), &r.get_rect()),
                vec![Rc::new(l), Rc::new(r)],
            )
        }
    }

    #[inline]
    pub(self) fn split(&self) -> (Self, Self) {
        match self {
            Leaf(_, _) => panic!("illegal split on leaf"),
            LeafNode(r, v) => Self::_split(r, &mut v.clone()),
            InternalNode(r, v) => Self::_split(r, &mut v.clone()),
        }
    }

    fn _split(r: &Rect, v: &mut [Rc<RTree<T, M>>]) -> (Self, Self) {
        let mut min_axis = 0;
        let mut min_axis_margin = f64::INFINITY;
        let mut min_axis_best_split_index = 1;

        let m = core::cmp::max(1, M * 2 / 5);

        for axis in 0..r.dim() {
            v.sort_by_key(|t| t.get_rect().0[axis]);

            let mut best_split_index = m;
            let mut best_margin_overlap = (f64::INFINITY, f64::INFINITY);
            let mut total_margin = 0.;
            for i in 1..=(M - 2 * m + 2) {
                let (l, r) = v.split_at(m - 1 + i);
                // first_bbox does not actually need to be recomputed entirely each time but osef
                let first_bbox = l
                    .iter()
                    .map(|t| t.get_rect())
                    .fold(v[0].get_rect(), |r1, r2| Rect::merge(&r1, &r2));
                let second_bbox = r
                    .iter()
                    .map(|t| t.get_rect())
                    .fold(v[0].get_rect(), |r1, r2| Rect::merge(&r1, &r2));

                let margin = first_bbox.margin() + second_bbox.margin().into_inner();
                // println!("{:?}", first_bbox.inter(second_bbox.clone()));
                let overlap = first_bbox.inter(&second_bbox).volume().into_inner();

                if (margin, overlap) <= best_margin_overlap {
                    best_margin_overlap = (margin, overlap);
                    best_split_index = m - 1 + i;
                }
                total_margin += margin;
            }

            if total_margin <= min_axis_margin {
                min_axis_margin = total_margin;
                min_axis = axis;
                min_axis_best_split_index = best_split_index;
            }
        }

        v.sort_by_key(|t| t.get_rect().0[min_axis]);
        let (l, r) = v.split_at(min_axis_best_split_index);
        (
            Self::from_vec(l.iter().map(Rc::clone).collect()),
            Self::from_vec(r.iter().map(Rc::clone).collect()),
        )
    }

    #[inline]
    fn from_vec(v: Vec<Rc<RTree<T, M>>>) -> Self {
        let bb = v
            .iter()
            .map(|t| t.get_rect())
            .fold(v[0].get_rect(), |r1, r2| Rect::merge(&r1, &r2));
        match *v[0] {
            Leaf(_, _) => LeafNode(bb.clone(), v),
            _ => InternalNode(bb.clone(), v),
        }
    }

    #[inline]
    pub fn dim(&self) -> usize {
        match self {
            Leaf(p, _) => p.len(),
            LeafNode(r, _) => r.dim(),
            InternalNode(r, _) => r.dim(),
        }
    }
}

struct WeightedTree<T: Clone + std::fmt::Debug, const M: usize> {
    key: NotNan<f64>,
    value: Rc<RTree<T, M>>,
}
impl<T: Clone + fmt::Debug, const M: usize> PartialEq for WeightedTree<T, M> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T: Clone + fmt::Debug, const M: usize> Eq for WeightedTree<T, M> {}

impl<T: Clone + std::fmt::Debug, const M: usize> core::cmp::PartialOrd for WeightedTree<T, M> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + fmt::Debug, const M: usize> Ord for WeightedTree<T, M> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.key.cmp(&other.key) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
            std::cmp::Ordering::Equal => {
                if Rc::ptr_eq(&self.value, &other.value) {
                    std::cmp::Ordering::Equal
                } else {
                    (self.value.get_rect().volume(), Rc::as_ptr(&self.value))
                        .cmp(&(other.value.get_rect().volume(), Rc::as_ptr(&other.value)))
                        .reverse()
                }
            }
        }
    }
}

fn points_distance(x: &[f64], y: &[f64]) -> f64 {
    debug_assert_eq!(x.len(), y.len());
    x.iter()
        .zip(y)
        .map(|(&a, b)| (a - b) * (a - b))
        .fold(0., |a, b| a + b)
}
