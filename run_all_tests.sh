echo "Starting Square Snapshot Run 1"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_snapshot.wav --snapshot-method --single-ir --irfile=cube_snapshot.csv > cube_snapshot_1.log
echo "Starting Square Snapshot Run 2"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_snapshot.wav --snapshot-method --single-ir --irfile=cube_snapshot.csv > cube_snapshot_2.log
echo "Starting Square Snapshot Run 3"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_snapshot.wav --snapshot-method --single-ir --irfile=cube_snapshot.csv > cube_snapshot_3.log

echo "Starting Square Interp Run 1"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_interp.wav --single-ir --irfile=cube_interp.csv > cube_interp_1.log
echo "Starting Square Interp Run 2"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_interp.wav --single-ir --irfile=cube_interp.csv > cube_interp_2.log
echo "Starting Square Interp Run 3"
cargo run -r -- --fname=testfiles/speech.wav --scene=4 --rays=10000000 --scaling-factor=100 --outfile=cube_interp.wav --single-ir --irfile=cube_interp.csv > cube_interp_3.log

echo "Starting L Snapshot Run 1"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_snapshot.wav --snapshot-method --single-ir --irfile=l_snapshot.csv > l_snapshot_1.log
echo "Starting L Snapshot Run 2"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_snapshot.wav --snapshot-method --single-ir --irfile=l_snapshot.csv > l_snapshot_2.log
echo "Starting L Snapshot Run 3"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_snapshot.wav --snapshot-method --single-ir --irfile=l_snapshot.csv > l_snapshot_3.log

echo "Starting L Interp Run 1"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_interp.wav --single-ir --irfile=l_interp.csv > l_interp_1.log
echo "Starting L Interp Run 2"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_interp.wav --single-ir --irfile=l_interp.csv > l_interp_2.log
echo "Starting L Interp Run 3"
cargo run -r -- --fname=testfiles/speech.wav --scene=5 --rays=10000000 --scaling-factor=100 --outfile=l_interp.wav --single-ir --irfile=l_interp.csv > l_interp_3.log
echo "All done!"
