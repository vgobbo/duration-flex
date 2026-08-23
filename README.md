# Duration Flex

![rust.yml](https://github.com/vgobbo/duration-flex/actions/workflows/rust.yml/badge.svg)

Helper class to make it easier to specify durations. Specially useful in configuration files.

It is common for durations to be specified in configuration files as "the number of seconds", which might not be very readable in some cases.

This crate aims to help solving this problem by allowing the time unit to be specified alongside the amount of time.

**Example:**
- 1 hour and 23 minutes: `1h23m`
- 1 week, 6 days, 23 hours, 49 minutes and 50 seconds: `1w6d23h49m59s`
- 1 year, 2 weeks and 3 days: `1y2w3d`

**Supported Time Units**
- Years: `1y` (1 year, equivalent to 365 days).
- Weeks: `2w` (2 weeks).
- Days: `3d` (3 days).
- Hours: `15h` (15 hours).
- Minutes: `5m` (5 minutes).
- Seconds: `30s` (30 seconds).

> **Note:** Months are not supported because they vary in the amount of days (28 to 31 days). It is best to specify the desired duration in days instead (e.g. `30d`).

## Usage

Simply call one of the `from` methods to create an instance:
```rust
use duration_flex::DurationFlex;

pub fn main() {
    let df = DurationFlex::try_from("1w6d23h49m59s").unwrap();
    println!("{df}");
}
```

## Features
- `clap`: enable clap support, so it can be used as application arguments.
- `serde`: enable serde support.
- `utoipa`: enable support for the `utoipa` crate.
- `validator`: enable support for the `validator` crate.

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