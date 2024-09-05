use honeycomb::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect(
        "No input geometry specified - you can pass a path to a vtk input as command line argument",
    );
    let clip = args
        .get(2)
        .map(|val| match val.as_ref() {
            "left" => grisubal::Clip::Left,
            "right" => grisubal::Clip::Right,
            _ => {
                println!("W: unrecognised clip argument - running kernel without clipping");
                grisubal::Clip::None
            }
        })
        .unwrap_or_default();

    let map = grisubal::grisubal::<f64>(path, [1., 1.], clip).unwrap();

    std::hint::black_box(map);
}
