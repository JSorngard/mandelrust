use mandellib::{render, Frame, RenderParameters};

fn main() {
    let p = RenderParameters::new(
        10.try_into().unwrap(),
        10.try_into().unwrap(),
        255.try_into().unwrap(),
        3.try_into().unwrap(),
        false,
    ).unwrap();
    let f = Frame::new(-0.75, 0.0, 1.0, 1.0);
    println!("{:?}", render(p, f, false));
}
