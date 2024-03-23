# MobileGeometricAudioDemo

This is a proof-of-concept created in the process of [my bachelor's thesis](https://github.com/twynb/BachelorThesis) on geometric acoustics simulation in moving scenes.
A link to the thesis itself will be provided [here](#mobilegeometricaudiodemo) once it is published.

Note that only an energetic response is calculated - while audio is auralized using it, this is only to be taken as a rough example and not at all an accurate simulation.

## Usage

To run this app, either download it through the releases section or clone and build it yourself.
The following command line arguments are supported:

- `--fname=NAME`: The file name of the audio (in .wav format) to apply the resulting energetic response to. Required.
- `--scene=0`: The scene to simulate. The supported scenes are listed below. Required.
- `--rays=100000`: The number of rays to simulate per energetic response. Defaults to 100000.
- `--scaling-factor=10000`: Scale up the auralized audio's amplitude by this factor. Defaults to 10000.
- `--snapshot-method`: If set, run the simulation using the snapshot rather than the interpolated method.
- `--single-ir`: If set, only calculate a single impulse response at time 0 and apply it to the entire audio.
- `--outfile=NAME`: The file name to write the resulting audio to. Defaults to "result.wav".
- `--irfile=NAME`: If set, the energetic response is written in CSV format to this file.

To reproduce the tests from the bachelor thesis, install `cargo`/the rust toolchain,
then run `run_all_tests.sh` and `run_scene_1.sh`.

## Scenes

- 0: Static 4x4x3 cube scene, with the receiver in the middle and the emitter above the receiver.
- 1: Receiver sits 343.2m away from the emitter, the emitter only emits rays in the receiver's direction.
- 2: Scene 1, but the receiver moves towards the emitter at 1/9th the speed of sound.
- 3: Scene 2, but the receiver starts 4x as far away from the emitter.
- 4: Scene 0, but rotating once per second.
- 5: L-Shaped room rotating around one of its ends, with the receiver in the rotation axis and the emitter above the receiver.
