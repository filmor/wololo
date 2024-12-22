default: run

# Build
build:
    cargo build

# Run
run *args:
    RUST_LOG=info cargo run -- {{args}}

# Test
test:
    cargo test

# Build, run, and clean up the temporary image
run-container *args:
    image_id=$(podman build -q .) && \
    podman run --rm $image_id {{args}} && \
    podman rmi $image_id

