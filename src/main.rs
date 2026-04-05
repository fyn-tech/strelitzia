use strelitzia::trees::kd_tree::KDTree;
use strelitzia::multiarray::Vector;

fn main() {
    println!("Strelitzia computational physics library.");
    println!("Use as a library: `use strelitzia::prelude::*;`");

    draw_test();
}

fn draw_test() {
    let points: Vec<Vector<i32, 2>> = vec![
        Vector::<i32, 2>::new(5, -2),
        Vector::<i32, 2>::new(1, -4),
        Vector::<i32, 2>::new(3, 0),
    ];
    let tree = KDTree::new().build(&points, 3);
    tree.draw("kd_tree.svg").unwrap();
}