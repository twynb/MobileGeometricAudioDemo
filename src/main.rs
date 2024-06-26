use std::io::Write;
use std::time::Instant;

use demo::{ray::DEFAULT_PROPAGATION_SPEED, scene::SceneData, scene_builder};

const DEFAULT_NUMBER_OF_RAYS: u32 = 100000;
const DEFAULT_SCALING_FACTOR: f64 = 10000f64;

#[allow(clippy::too_many_lines)]
fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = std::env::args().collect();

    let mut input_fname: Option<&str> = None;
    let mut scene_index: Option<u32> = None;
    let mut number_of_rays: u32 = DEFAULT_NUMBER_OF_RAYS;
    let mut scaling_factor: f64 = DEFAULT_SCALING_FACTOR;
    let mut do_snapshot_method: bool = false;
    let mut single_ir: bool = false;
    let mut out_fname: &str = "result.wav";
    let mut ir_fname: Option<&str> = None;

    for arg in args.iter().skip(1) {
        let arg_split: Vec<&str> = arg.split('=').collect();
        match arg_split[0] {
            "--fname" => input_fname = Some(arg_split[1]),
            "--scene" => scene_index = arg_split[1].parse::<u32>().ok(),
            "--rays" => {
                number_of_rays = arg_split[1]
                    .parse::<u32>()
                    .unwrap_or_else(|_| panic!("\"--rays\" needs to be passed a number!"));
            }
            "--scaling-factor" => {
                scaling_factor = arg_split[1]
                    .parse::<f64>()
                    .unwrap_or_else(|_| panic!("\"--rays\" needs to be passed a number!"));
            }
            "--snapshot-method" => do_snapshot_method = true,
            "--single-ir" => single_ir = true,
            "--outfile" => out_fname = arg_split[1],
            "--irfile" => ir_fname = Some(arg_split[1]),
            _ => panic!("Unknown argument {}", arg_split[0]),
        };
    }

    let Some(input_fname) = input_fname else {
        panic!("Please provide a file name using \"--fname=FILENAME\"!")
    };
    let mut input_file = std::fs::File::open(std::path::Path::new(input_fname))
        .unwrap_or_else(|_| panic!("Input file couldn't be opened!"));
    let (header, input_data) = wav::read(&mut input_file)
        .unwrap_or_else(|_| panic!("An error occurred while parsing the input file!"));
    let input_sound_len: usize = if single_ir {
        1
    } else {
        match &input_data {
            wav::BitDepth::Eight(data) => data.len(),
            wav::BitDepth::Sixteen(data) => data.len(),
            wav::BitDepth::TwentyFour(data) => data.len(),
            wav::BitDepth::ThirtyTwoFloat(data) => data.len(),
            wav::BitDepth::Empty => panic!("Input file did not contain any data!"),
        }
    };

    let Some(scene_index) = scene_index else {
        println!("Please provide a valid scene index using \"--scene=INDEX\"! The following scene indices are supported:");
        print_supported_scenes();
        panic!();
    };
    let scene = match scene_index {
        0 => scene_builder::static_cube_scene(),
        1 => scene_builder::static_receiver_scene(),
        2 => scene_builder::approaching_receiver_scene(header.sampling_rate),
        3 => scene_builder::long_approaching_receiver_scene(header.sampling_rate),
        4 => scene_builder::rotating_cube_scene(header.sampling_rate),
        5 => scene_builder::rotating_l_scene(header.sampling_rate),
        _ => {
            println!("Invalid scene index! The following scene indices are supported:");
            print_supported_scenes();
            panic!();
        }
    };
    let scene_name = match scene_index {
        0 => "static cube",
        1 => "static receiver",
        2 => "approaching receiver 1s",
        3 => "approaching receiver 4s",
        4 => "rotating cube 1s",
        5 => "rotating L 1s",
        _ => "error",
    };
    println!("Selected scene #{scene_index}: \"{scene_name}\".");
    let scene_data = SceneData::<typenum::U10>::create_for_scene(scene);

    println!("Calculating and applying {input_sound_len} impulse responses with {number_of_rays} rays each, this will take a loooong while...");
    let time_start = Instant::now();
    let (result, impulse_response) = scene_data.simulate_for_time_span(
        &input_data,
        number_of_rays,
        DEFAULT_PROPAGATION_SPEED,
        f64::from(header.sampling_rate),
        scaling_factor,
        do_snapshot_method,
        single_ir,
    );
    let elapsed = time_start.elapsed().as_secs();
    println!(
        "Finished calculation in {}:{:02}:{:02}",
        elapsed / 3600,
        (elapsed % 3600) / 60,
        elapsed % 60
    );

    println!(
        "T60: {}",
        impulse_response.len() as f64 / f64::from(header.sampling_rate)
    );

    let mut output_file = std::fs::File::create(std::path::Path::new(out_fname))
        .unwrap_or_else(|_| panic!("Output file couldn't be opened!"));
    wav::write(header, &result, &mut output_file)
        .unwrap_or_else(|_| panic!("Output file couldn't be written to!"));

    match ir_fname {
        Some(fname) => {
            let mut ir_file = std::fs::File::create(std::path::Path::new(fname))
                .unwrap_or_else(|_| panic!("IR Output file couldn't be opened!"));
            for value in impulse_response {
                write!(ir_file, "{value};")
                    .unwrap_or_else(|_| panic!("Couldn't write impulse response!"));
            }
        }
        None => (),
    }
}

/// Print out all supported scene indices.
fn print_supported_scenes() {
    println!("\t0 - Static Cube");
    println!("\t1 - Static Receiver");
    println!("\t2 - Approaching Receiver 1s");
    println!("\t3 - Approaching Receiver 4s");
    println!("\t4 - Rotating Cube 1s");
    println!("\t5 - Rotating L 1s");
}
