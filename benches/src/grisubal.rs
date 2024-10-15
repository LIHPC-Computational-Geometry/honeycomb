use honeycomb::kernels::grisubal;

fn main() {
    // read file path, grid sizes, and clip policy from the command line
    // only file path is required
    let (path, clip, lx, ly) = {
        let args: Vec<String> = std::env::args().collect();
        match (args.get(1), args.get(2), args.get(3), args.get(4)) {
            (None, _, _, _) => panic!("E: No input geometry specified - you can pass a path to a vtk input as command line argument"),
            t => {
                (
                    t.0.unwrap().clone(),
                    t.3.map(|val| match val.as_ref() {
                        "left" => grisubal::Clip::Left,
                        "right" => grisubal::Clip::Right,
                        _ => {
                            eprintln!("W: unrecognised clip argument - running kernel without clipping");
                            grisubal::Clip::None
                        }
                    }).unwrap_or_default(),
                    t.1.map(|val| val.parse::<f64>().unwrap_or(1.)).unwrap(),
                    t.2.map(|val| val.parse::<f64>().unwrap_or(1.)).unwrap(),
                )
            }
        }
    };

    let map = grisubal::grisubal::<f64>(path, [lx, ly], clip).unwrap();

    std::hint::black_box(map);
}
