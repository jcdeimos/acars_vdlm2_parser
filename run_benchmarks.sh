# /bin/sh

echo "Running benchmarks"
echo "These tests will only fail if file loading is unsuccessful."
echo "---"
cargo bench --bench message_processing -- --verbose
echo "---"
echo "Tests completed."