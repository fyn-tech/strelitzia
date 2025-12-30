// use strelitzia::prelude::*;

pub type Real = f64;

pub trait BasePredicates {
    fn orient_2d(pa: &Self, pb: &Self, pc: &Self) -> Real;

    fn orient_3d(pa: &Self, pb: &Self, pc: &Self, pd: &Self) -> Real;

    fn in_circle(pa: &Self, pb: &Self, pc: &Self, pd: &Self) -> Real;

    fn in_sphere(pa: &Self, pb: &Self, pc: &Self, pd: &Self, pe: &Self) -> Real;
}

fn main() {
    strelitzia::run();
}
