echo "Starting Snapshot run"
cargo run -r -- --fname=testfiles/sine_440Hz_1second.wav --scene=2 --rays=1 --scaling-factor=1 --snapshot-method --outfile=approach_snap.wav > approach_snap.log
echo "Starting Interp run"
cargo run -r -- --fname=testfiles/sine_440Hz_1second.wav --scene=2 --rays=1 --scaling-factor=1 --outfile=approach_interp.wav > approach_interp.log