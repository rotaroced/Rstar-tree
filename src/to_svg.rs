use crate::rect::*;
use crate::rtree::RTree;
use svg::Document;
use svg::node::element::{Circle, Rectangle};
const COLORS: [&str; 8] = [
    "red",
    "orange",
    // "goldenrod",
    "darkgreen",
    // "green",
    "cyan",
    // "lightskyblue",
    "blue",
    // "darkslateblue",
    "magenta",
    // "mediumorchid",
    "maroon",
    // "grey",
    "black",
];

fn rect_to_svg(t: &Rect<2>, depth: usize) -> Rectangle {
    Rectangle::new()
        .set("x", t.0[0].0.into_inner() - 0.1 * depth as f64)
        .set("y", t.0[1].0.into_inner() - 0.1 * depth as f64)
        .set(
            "width",
            (t.0[0].1 - t.0[0].0).into_inner() + 0.2 * depth as f64,
        )
        .set(
            "height",
            (t.0[1].1 - t.0[1].0).into_inner() + 0.2 * depth as f64,
        )
        .set("stroke", COLORS[depth])
        .set("stroke-width", 1)
        .set("fill", "none")
}

pub fn point_to_svg(p: &[f64]) -> Circle {
    assert_eq!(p.len(), 2);
    Circle::new()
        .set("cx", p[0])
        .set("cy", p[1])
        .set("fill", "black")
        .set("color", "black")
        .set("r", 1)
}

fn rects_to_svg<T: Clone + std::fmt::Debug, const M: usize>(
    t: &RTree<T, M, 2>,
    depth: usize,
    mut d: Document,
) -> Document {
    match t {
        RTree::Leaf(_, _) => d,
        RTree::InternalNode(r, v) => {
            d = d.clone().add(rect_to_svg(r, depth));
            for a in v.iter().flatten() {
                d = rects_to_svg(a, depth + 1, d);
            }
            d
        }
        RTree::LeafNode(r, _) => d.add(rect_to_svg(r, depth)),
    }
}

fn add_points<T: Clone + std::fmt::Debug, const M: usize>(
    t: &RTree<T, M, 2>,
    d: Document,
) -> Document {
    match t {
        RTree::Leaf(p, _) => d.add(point_to_svg(p)),
        RTree::InternalNode(_, v) => v
            .iter()
            .fold(d, |d, a| if let Some(t) = a { add_points(t, d) } else { d }),
        RTree::LeafNode(_, v) => v.iter().flatten().fold(d, |d, a| add_points(a, d)),
    }
}

pub fn bidimensional_rtree_to_svg<T: Clone + std::fmt::Debug, const M: usize>(
    t: RTree<T, M, 2>,
) -> Document {
    add_points(
        &t,
        rects_to_svg(
            &t,
            0,
            Document::new()
                .set("width", t.get_rect().0[0].1.into_inner() + 14.)
                .set("height", t.get_rect().0[1].1.into_inner() + 14.),
        ),
    )
}
