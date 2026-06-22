podman run --rm -it \
  -v /home/seev0/num/1-c0d3/rust:/home/seev0/etc:Z \
  -e CARGO_TARGET_DIR=/home/seev0/etc/tmp \
  alpine-rust-builder:latest \
  cargo build --release --manifest-path /home/seev0/etc/trade/Cargo.toml -p usta
