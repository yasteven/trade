podman run --rm -it \
  -v /home/seev0/num/1-c0d3/rust:/home/seev0/etc:Z \
  -e CARGO_TARGET_DIR=/home/seev0/etc/trade/bin/alpine_usta/ \
  alpine-rust-builder:latest \
  cargo build --release --manifest-path /home/seev0/etc/trade/Cargo.toml -p usta

scp {bin/alpine_usta/release/*.pem,bin/alpine_usta/release/usta} root@192.168.122.239:/var/trade/usta
