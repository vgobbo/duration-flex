# Duration Flex

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