# Duration Flex

Helper class to make it easier to specify durations. Specially useful in configuration files.

It is common for durations to be specified in configuration files as "the number of seconds", which might not be very readable in some cases.

This crate aims to help solving this problem by allowing the time unit to be specified alongside the amount of time.

**Example:**
- 1 hour and 23 minutes: `1h23m`
- 1 week, 6 days, 23 hours, 49 minutes andd 50 seconds: `1w6d23h49m59s`

## Features
- `clap`: enable clap support, so it can be used as application arguments.
- `serde`: enable serde support.

## Developing

1. Install fish shell.
2. Install a recent (1.80+) rust compiler (with cargo).
3. Install a toolchain compatible with the desired target, like `stable-aarch64-apple-darwin`.
```shell
rustup toolchain install stable-aarch64-apple-darwin
```
4. Install a nightly profile compatible with the current machine, like:
```shell
rustup toolchain install nightly-aarch64-apple-darwin
```
5. Install rust packages:
```shell
cargo install --profile release taplo ripgrep
```
6. Setup hooks: `./scripts/setup-hooks`
7. (**Optional**) Run `./scripts/setup-target` to initialize `target/` in the temporary directory. This has to be done everytime the machine is restarted. 

To test, always specify `--all-features`:
```shell
cargo test --all-features
```