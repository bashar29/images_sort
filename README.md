## About

Application to sort your photos.


```
cargo run -- --source-dir photos --dest-dir out
```


### Cross compile (from my M2 to a NAS under linux)
```
rustup target add x86_64-unknown-linux-musl
export CC=x86_64-linux-musl-gcc
RUSTFLAGS="-C linker=x86_64-linux-musl-gcc" cargo build --release --target x86_64-unknown-linux-musl
```
# Not tested :
```
rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C link-arg=-s' cargo build --release --target x86_64-unknown-linux-musl
```
