![](docs/logo.png)

memflow - machine introspection framework

#

## building
- install capnp package (soon to be removed for default builds)
- run `cargo build --release --all --all-features --examples` to build everything
- run `cargo test --all --all-features` to run all tests
- run ... to run all benchmarks
- run `cargo clippy --all-targets --all-features -- -D warnings` to run clippy linting on everything

#

## usage
- run `cargo run --release -- -c qemu_procfs -vvv` to run the cli on qemu
