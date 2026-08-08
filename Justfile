default: dev

# Start the Topcoat development server with asset bundling and file watching.
dev:
    RUST_LOG=info topcoat dev --bin wololo

# Build the application and its Topcoat asset bundle.
build:
    cargo build --bin wololo
    topcoat asset bundle --bin wololo

# Build the application and asset bundle, then run with optional application arguments.
run *args:
    just build
    RUST_LOG=info cargo run -- {{args}}

# Test
test:
    cargo test

# Build, run, and clean up the temporary image
run-container *args:
    image_id=$(podman build -q .) && \
    podman run --rm $image_id {{args}} && \
    podman rmi $image_id
