#![feature(test)]

extern crate test;
extern crate wasm_gameoflife;
use wasm_gameoflife::universe::Universe;

#[bench]
fn universe_ticks_64(b: &mut test::Bencher) {
    let mut universe = Universe::new(64, 64);
    universe.randomize();

    b.iter(|| {
        universe.step();
    });
}

#[bench]
fn universe_ticks_256(b: &mut test::Bencher) {
    let mut universe = Universe::new(256, 256);
    universe.randomize();

    b.iter(|| {
        universe.step();
    });
}

#[bench]
fn universe_ticks_512(b: &mut test::Bencher) {
    let mut universe = Universe::new(512, 512);
    universe.randomize();

    b.iter(|| {
        universe.step();
    });
}

#[bench]
fn universe_ticks_1024(b: &mut test::Bencher) {
    let mut universe = Universe::new(1024, 1024);
    universe.randomize();

    b.iter(|| {
        universe.step();
    });
}
