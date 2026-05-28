use super::rect::*;
use std::collections::BinaryHeap;
use std::fmt;
use std::rc::Rc;

/// M is the maximum number of children per node.
/// algorithms from this paper : https://dl.acm.org/doi/pdf/10.1145/93605.98741
#[derive(Clone, Debug)]
pub enum RTree<T: fmt::Debug + Clone, const M: usize, const DIM: usize> {
    InternalNode(Rect<DIM>, [Option<Rc<RTree<T, M, DIM>>>; M]),
    LeafNode(Rect<DIM>, [Option<Rc<RTree<T, M, DIM>>>; M]),
    Leaf([f64; DIM], Rc<T>),
}

use RTree::*;

impl<T: fmt::Debug + Clone, const M: usize, const DIM: usize> Default for RTree<T, M, DIM> {
    #[inline]
    fn default() -> RTree<T, M, DIM> {
        LeafNode(Rect::empty_rect(), [const { None }; M])
    }
}

impl<T: fmt::Debug + Clone, const M: usize, const DIM: usize> RTree<T, M, DIM> {
    #[inline]
    pub fn get_rect(&self) -> Rect<DIM> {
        match self {
            Self::InternalNode(r, _) => *r,
            Self::LeafNode(r, _) => *r,
            Self::Leaf(p, _) => Rect::from_point(p),
        }
    }

    /// returns a tuple containing :
    /// - the closest point of the R-tree
    /// - the distance to this point
    /// - the value of this point
    pub fn closest(&self, p: &[f64; DIM]) -> ([f64; DIM], f64, Rc<T>) {
        let mut pq = BinaryHeap::new();

        match self {
            Leaf(_, _) => unreachable!(),
            LeafNode(_, v) => {
                for t in v {
                    if t.is_none() {
                        break;
                    }
                    let unwrapped = t.as_ref().unwrap();
                    pq.push(WeightedTree {
                        key: unwrapped.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(unwrapped),
                    });
                }
            }
            InternalNode(_, v) => {
                for t in v {
                    if t.is_none() {
                        break;
                    }
                    let unwrapped = t.as_ref().unwrap();
                    pq.push(WeightedTree {
                        key: unwrapped.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(unwrapped),
                    });
                }
            }
        }

        loop {
            let WeightedTree { key, value } = pq.pop().unwrap();

            match value.as_ref() {
                Leaf(x, e) => {
                    return (*x, points_distance(x, p), Rc::clone(e));
                }
                LeafNode(_, v) => {
                    for t in v {
                        if t.is_none() {
                            break;
                        }
                        let unwrapped = t.as_ref().unwrap();
                        pq.push(WeightedTree {
                            key: unwrapped.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(unwrapped),
                        });
                    }
                }
                InternalNode(_, v) => {
                    for t in v {
                        if t.is_none() {
                            break;
                        }
                        let unwrapped = t.as_ref().unwrap();
                        pq.push(WeightedTree {
                            key: unwrapped.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(unwrapped),
                        });
                    }
                }
            }
        }
    }

    pub fn k_closest(&self, p: &[f64; DIM], mut k: usize) -> Vec<([f64; DIM], NotNan<f64>, Rc<T>)> {
        let mut pq = BinaryHeap::new();

        match self {
            Leaf(_, _) => unreachable!(),
            LeafNode(_, v) => {
                for t in v.iter().flatten() {
                    pq.push(WeightedTree {
                        key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                        value: Rc::clone(t),
                    });
                }
            }
            InternalNode(_, v) => {
                for t in v.iter().flatten() {
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
                    res.push((*x, points_distance(x, p).try_into().unwrap(), Rc::clone(e)));
                    k -= 1
                }
                LeafNode(_, v) => {
                    for t in v.iter().flatten() {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        });
                    }
                }
                InternalNode(_, v) => {
                    for t in v.iter().flatten() {
                        pq.push(WeightedTree {
                            key: t.get_rect().dist_from_point(p).try_into().unwrap(),
                            value: Rc::clone(t),
                        });
                    }
                }
            }
        }

        res
    }

    pub(crate) fn n_taken<A>(v: &[Option<A>; M]) -> usize {
        let mut a = 0;
        let mut b = M;

        while b - a > 1 {
            let m = (a + b) / 2;
            if v[m].is_none() {
                b = m;
            } else {
                a = m;
            }
        }

        if v[a].is_none() { a } else { a + 1 }
    }

    /// inserts a node and returns Some((l, r)) when the node is split.
    pub(crate) fn _insert(&mut self, p: &[f64; DIM], value: T) -> Option<(Self, Self)> {
        match self {
            Leaf(_, _) => panic!("tried illegal insertion in a leaf"),
            LeafNode(r, v) => {
                let n = Self::n_taken(v);
                if n >= M {
                    Some(Self::split(v, &Leaf(*p, Rc::new(value))))
                } else {
                    v[n] = Some(Rc::new(Leaf(*p, Rc::new(value))));
                    if v.len() == 1 {
                        *r = Rect::from_point(p);
                    }
                    *r = Rect::merge(r, &Rect::from_point(p));
                    None
                }
            }
            InternalNode(bb, v) => {
                let n = Self::n_taken(v);

                *bb = Rect::merge(bb, &Rect::from_point(p));

                let mut min_i = 0;
                let mut min_enlargement = f64::INFINITY;
                let mut min_rect_volume = 0.;

                for i in 0..DIM {
                    if let Some(child) = &v[i] {
                        let r = child.get_rect();
                        let enlargement =
                            Rect::merge(&r, &Rect::from_point(p)).volume().into_inner()
                                - r.volume().into_inner();
                        // resolves ties using volume
                        // println!("{child:?}, {enlargement} {:?}\n", r.volume());
                        if (enlargement, r.volume().into_inner())
                            <= (min_enlargement, min_rect_volume)
                        {
                            min_i = i;
                            min_enlargement = enlargement;
                            min_rect_volume = r.volume().into_inner();
                        }
                    }
                }

                let res = Self::_insert(Rc::make_mut(v[min_i].as_mut().unwrap()), p, value);
                if let Some((l, r)) = res {
                    v[min_i] = Some(Rc::new(l));
                    if n >= M {
                        return Some(Self::split(v, &r));
                    }
                    v[n] = Some(Rc::new(r));
                }

                None
            }
        }
    }

    #[inline]
    pub fn insert(&mut self, p: &[f64; DIM], value: T) {
        if let Some((l, r)) = self._insert(p, value) {
            *self = Self::InternalNode(Rect::merge(&l.get_rect(), &r.get_rect()), {
                let mut children = [const { None }; M];
                children[0] = Some(Rc::new(l));
                children[1] = Some(Rc::new(r));
                children
            });
        }
    }

    #[inline]
    pub(self) fn split(vec: &[Option<Rc<Self>>; M], t: &Self) -> (Self, Self) {
        let mut w = if cfg!(debug_assertions) {
            vec.iter().map(|x| x.clone().unwrap()).collect::<Vec<_>>()
        } else {
            vec.iter()
                .map(|x| unsafe { x.clone().unwrap_unchecked() })
                .collect::<Vec<_>>()
        };
        w.push(Rc::from(t.clone()));
        let mut min_axis = 0;
        let mut min_axis_margin = f64::INFINITY;
        let mut min_axis_best_split_index = 1;

        let m = core::cmp::max(1, M * 2 / 5);

        for axis in 0..DIM {
            w.sort_unstable_by_key(|t| t.get_rect().0[axis]);

            let mut best_split_index = m;
            let mut best_margin_overlap = (f64::INFINITY, f64::INFINITY);
            let mut total_margin = 0.;
            let mut first_bbox = w[0].get_rect();
            for i in 1..=(M - 2 * m + 2) {
                first_bbox = Rect::merge(&first_bbox, &w[m - 1 + i - 1].get_rect());
                let second_bbox = ((m - 1 + i)..w.len())
                    .map(|j| w[j].get_rect())
                    .fold(w[0].get_rect(), |r1, r2| Rect::merge(&r1, &r2));

                let margin = first_bbox.margin() + second_bbox.margin().into_inner();
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

        w.sort_by_key(|t| t.get_rect().0[min_axis]);
        let (l, r) = w.split_at(min_axis_best_split_index);
        (
            Self::from_vec(l.iter().map(Rc::clone).collect()),
            Self::from_vec(r.iter().map(Rc::clone).collect()),
        )
    }

    #[inline]
    fn from_vec(v: Vec<Rc<RTree<T, M, DIM>>>) -> Self {
        if v.is_empty() {
            return Self::default();
        }

        let bb = v
            .iter()
            .map(|t| t.get_rect())
            .fold(v[0].get_rect(), |r1, r2| Rect::merge(&r1, &r2));

        // TODO: check to know whether Leaf node or internal node ...
        let mut children = [const { None }; M];
        for i in 0..v.len() {
            children[i] = Some(Rc::clone(&v[i]));
        }

        let leaf = v.iter().all(|x| x.is_leaf());
        if leaf {
            LeafNode(bb, children)
        } else {
            debug_assert!(v.iter().all(|x| !x.is_leaf()));
            InternalNode(bb, children)
        }
    }

    pub fn is_leaf(&self) -> bool {
        matches!(self, Leaf(_, _))
    }

    pub fn is_internal_node(&self) -> bool {
        matches!(self, InternalNode(_, _))
    }
}

struct WeightedTree<T: Clone + std::fmt::Debug, const M: usize, const DIM: usize> {
    key: NotNan<f64>,
    value: Rc<RTree<T, M, DIM>>,
}
impl<T: Clone + fmt::Debug, const M: usize, const DIM: usize> PartialEq
    for WeightedTree<T, M, DIM>
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && Rc::ptr_eq(&self.value, &other.value)
    }
}

impl<T: Clone + fmt::Debug, const M: usize, const DIM: usize> Eq for WeightedTree<T, M, DIM> {}

impl<T: Clone + std::fmt::Debug, const M: usize, const DIM: usize> core::cmp::PartialOrd
    for WeightedTree<T, M, DIM>
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone + fmt::Debug, const M: usize, const DIM: usize> Ord for WeightedTree<T, M, DIM> {
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

pub fn points_distance<const DIM: usize>(x: &[f64; DIM], y: &[f64; DIM]) -> f64 {
    x.iter()
        .zip(y)
        .map(|(&a, b)| (a - b) * (a - b))
        .fold(0., |a, b| a + b)
}
