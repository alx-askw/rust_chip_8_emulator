#[allow(dead_code, unused_variables)]

enum First {
    PixelOn(bool),
}

struct Second {
    pixel_on: bool,
}

fn main() {
    let tst_one = "test string";
    println!("here {:p}", &tst_one);
    let bool_test: First = First::PixelOn(true);
}
