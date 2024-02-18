use std::time::Instant;

use demo::{ray::DEFAULT_PROPAGATION_SPEED, scene::SceneData, scene_builder};

const DEFAULT_NUMBER_OF_RAYS: u32 = 100000;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() >= 3,
        "Please provide at least a scene index and an input file!"
    );

    let scene_index = args[1]
        .parse::<u32>()
        .unwrap_or_else(|_| panic!("Scene index must be a number!"));
    let scene = match scene_index {
        0 => scene_builder::static_cube_scene(),
        _ => panic!("Invalid scene index! Only scene index 0 is supported at this time."),
    };

    let input_fname = &args[2];
    let mut input_file = std::fs::File::open(std::path::Path::new(input_fname))
        .unwrap_or_else(|_| panic!("Input file couldn't be opened!"));
    let (header, input_data) = wav::read(&mut input_file)
        .unwrap_or_else(|_| panic!("An error occurred while parsing the input file!"));
    let input_sound_len: usize = match &input_data {
        wav::BitDepth::Eight(data) => data.len(),
        wav::BitDepth::Sixteen(data) => data.len(),
        wav::BitDepth::TwentyFour(data) => data.len(),
        wav::BitDepth::ThirtyTwoFloat(data) => data.len(),
        wav::BitDepth::Empty => panic!("Input file did not contain any data!"),
    };

    let mut number_of_rays = DEFAULT_NUMBER_OF_RAYS;
    if args.len() >= 4 {
        if let Result::Ok(value) = args[3].parse::<u32>() {
            number_of_rays = value;
        }
    }

    let scene_data = SceneData::<typenum::U10>::create_for_scene(scene);
    println!("Calculating and applying {input_sound_len} impulse responses with {number_of_rays} rays each, this will take a loooong while...");
    let time_start = Instant::now();
    let result = scene_data.simulate_for_time_span(
        &input_data,
        number_of_rays,
        DEFAULT_PROPAGATION_SPEED,
        header.sampling_rate as f32,
        number_of_rays as f32,
    );
    let elapsed = time_start.elapsed().as_secs();
    println!("Finished calculation in {}:{}:{}", elapsed / 3600, (elapsed % 3600) / 60, elapsed % 60);

    let mut output_file = std::fs::File::create(std::path::Path::new("result.wav"))
        .unwrap_or_else(|_| panic!("Output file couldn't be opened!"));
    wav::write(header, &result, &mut output_file)
        .unwrap_or_else(|_| panic!("Output file couldn't be written to!"));
}
