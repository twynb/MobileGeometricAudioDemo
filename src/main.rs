use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        mpsc::{self, Sender, TryRecvError},
        Arc,
    },
    thread::{self, sleep},
    time::Instant,
};

use demo::{ray::DEFAULT_PROPAGATION_SPEED, scene::SceneData, scene_builder};

const DEFAULT_NUMBER_OF_RAYS: u32 = 100000;
const DEFAULT_SCALING_FACTOR: f64 = 10000f64;

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");
    let args: Vec<String> = std::env::args().collect();

    let mut input_fname: Option<&str> = None;
    let mut scene_index: Option<u32> = None;
    let mut number_of_rays: u32 = DEFAULT_NUMBER_OF_RAYS;
    let mut scaling_factor: f64 = DEFAULT_SCALING_FACTOR;
    let mut do_snapshot_method: bool = false;
    let mut out_fname: &str = "result.wav";

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
            "--outfile" => out_fname = arg_split[1],
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
    let input_sound_len: usize = match &input_data {
        wav::BitDepth::Eight(data) => data.len(),
        wav::BitDepth::Sixteen(data) => data.len(),
        wav::BitDepth::TwentyFour(data) => data.len(),
        wav::BitDepth::ThirtyTwoFloat(data) => data.len(),
        wav::BitDepth::Empty => panic!("Input file did not contain any data!"),
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
        _ => "error",
    };
    println!("Selected scene #{scene_index}: \"{scene_name}\".");
    let loop_duration = scene.loop_duration;
    let scene_data = SceneData::<typenum::U10>::create_for_scene(scene);

    let progress_counter = Arc::new(AtomicU32::new(0));
    let kill_rx = spawn_progress_counter_thread(input_sound_len, &progress_counter, loop_duration);

    println!("Calculating and applying {input_sound_len} impulse responses with {number_of_rays} rays each, this will take a loooong while...");
    let time_start = Instant::now();
    let result = scene_data.simulate_for_time_span(
        &input_data,
        number_of_rays,
        DEFAULT_PROPAGATION_SPEED,
        f64::from(header.sampling_rate),
        1f64 / scaling_factor,
        do_snapshot_method,
        &progress_counter,
    );
    let elapsed = time_start.elapsed().as_secs();
    let _ = kill_rx.send(()); // we don't really care if this somehow fails
    println!(
        "Finished calculation in {}:{:02}:{:02}",
        elapsed / 3600,
        (elapsed % 3600) / 60,
        elapsed % 60
    );

    let mut output_file = std::fs::File::create(std::path::Path::new(out_fname))
        .unwrap_or_else(|_| panic!("Output file couldn't be opened!"));
    wav::write(header, &result, &mut output_file)
        .unwrap_or_else(|_| panic!("Output file couldn't be written to!"));
}

/// Print out all supported scene indices.
fn print_supported_scenes() {
    println!("\t0 - Static Cube");
    println!("\t1 - Static Receiver");
    println!("\t2 - Approaching Receiver 1s");
    println!("\t3 - Approaching Receiver 4s");
    println!("\t4 - Rotating Cube 1s");
}

/// Spawn a thread that repeatedly checks the progress counter and displays progress.
fn spawn_progress_counter_thread(
    input_sound_len: usize,
    progress_counter: &Arc<AtomicU32>,
    loop_duration: Option<u32>
) -> Sender<()> {
    let input_len = loop_duration.unwrap_or(input_sound_len as u32) as usize;
    let number_of_chunks = (input_len / 1000) as u32 + u32::from(input_len % 1000 != 0);
    let cloned_counter = Arc::clone(progress_counter);
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        let current_value = { cloned_counter.load(Ordering::Relaxed) };
        print!("\rFinished {current_value}/{number_of_chunks} Batches");
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                return;
            }
            Err(TryRecvError::Empty) => (),
        };
        sleep(std::time::Duration::from_millis(100));
    });
    tx
}
