# Duration Flex

Helper class to make it easier to specify durations. Specially useful in configuration files.

It is common for durations to be specified in configuration files as "the number of seconds", which might not be very readable in some cases.

This crate aims to help solving this problem by allowing the time unit to be specified alongside the amount of time.

**Example:**
- 1 hour and 23 minutes: `1h23m`
- 1 week, 6 days, 23 hours, 49 minutes andd 50 seconds: `1w6d23h49m59s`

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