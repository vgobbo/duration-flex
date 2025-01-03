//! # Duration Flex
//!
//! Helper to make it easier to specify duration files. Specially useful in configuration files.
//! - Basic interoperability with [`chrono::DateTime`], allowing it to be added/subbed from it.
//! - Can be built from [`chrono::Duration`].
//! - Can be built from [`std::time::Duration`].
//!
//! **Example:**
//! - 1 hour and 23 minutes: `1h23m`
//! - 1 week, 6 days, 23 hours, 49 minutes andd 50 seconds: `1w6d23h49m59s`
//!
//! **Supported Time Units**
//! - Weeks: `2w` (2 weeks).
//! - Days: `3d` (3 days).
//! - Hours: `15h` (15 hours).
//! - Minutes: `5m` (5 minutes).
//! - Seconds: `30s` (30 seconds).
//!
//! ## Usage
//!
//! Simply call one of the `from` methods to create an instance:
//! ```
//! use duration_flex::DurationFlex;
//!
//! # pub fn main() {
//! let df = DurationFlex::try_from("1w6d23h49m59s").unwrap();
//! println!("{df}");
//! # }
//! ```
//!
//! ## Features
//! - `clap`: enable clap support, so it can be used as application arguments.
//! - `serde`: enable serde support.

use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time;

use chrono::{DateTime, Duration, TimeZone};
#[cfg(feature = "clap")]
use clap::builder::OsStr;
use once_cell::sync::Lazy;
use regex::{Match, Regex};
#[cfg(feature = "serde")]
use serde::de::{Error, Unexpected, Visitor};
#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const SECS_PER_MINUTES: i64 = 60;
const SECS_PER_HOUR: i64 = 60 * SECS_PER_MINUTES;
const SECS_PER_DAY: i64 = 24 * SECS_PER_HOUR;
const SECS_PER_WEEK: i64 = 7 * SECS_PER_DAY;

/// Errors returned by the different methods.
#[derive(Copy, Clone, Debug)]
pub enum DurationFlexError {
	/// String format is not valid, e.g. `1y` (`y` is not supported).
	InvalidFormat,

	/// Value is out of range.
	OutOfRange,
}

#[allow(clippy::tabs_in_doc_comments)]
/// Type to conveniently specify durations and interoperate with [`chrono::Duration`].
///
/// The correct way of building this, is through one of the `from` methods.
///
/// With the `clap` feature, can be used with [`clap`]:
/// ```
/// use clap::Args;
/// use duration_flex::DurationFlex;
///
/// #[derive(Args)]
/// pub struct Arguments {
/// 	#[arg(long, default_value_t = Arguments::default().duration)]
/// 	duration: DurationFlex,
/// }
///
/// impl Default for Arguments {
/// 	fn default() -> Self {
/// 		Self { duration: DurationFlex::try_from("1w6d23h49m59s").unwrap() }
/// 	}
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct DurationFlex {
	secs: i64,
	nanos: i32,
}

static REGEX_STR: &str =
	r"^((?P<weeks>\d+)w)?((?P<days>\d+)d)?((?P<hours>\d+)h)?((?P<minutes>\d+)m)?((?P<seconds>\d+)s)?$";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(REGEX_STR).unwrap());

impl DurationFlex {
	/// Whole seconds.
	pub fn secs(&self) -> i64 {
		self.secs
	}

	/// Nano-seconds.
	pub fn nanos(&self) -> i32 {
		self.nanos
	}

	fn de_component(r#match: Match) -> i64 {
		r#match.as_str().parse().unwrap()
	}

	fn ser_component(secs: &mut i64, component: &str, component_secs: i64, f: &mut Formatter<'_>) -> std::fmt::Result {
		let value = *secs / component_secs;
		*secs -= value * component_secs;

		if value == 0 {
			Ok(())
		} else {
			write!(f, "{}{}", value, component)
		}
	}
}

impl Sub<Duration> for DurationFlex {
	type Output = Duration;

	fn sub(self, rhs: Duration) -> Self::Output {
		Duration::from(self) - rhs
	}
}

impl Add<Duration> for DurationFlex {
	type Output = Duration;

	fn add(self, rhs: Duration) -> Self::Output {
		Duration::from(self) + rhs
	}
}

impl<T> Add<DateTime<T>> for DurationFlex
where
	T: TimeZone,
{
	type Output = DateTime<T>;

	fn add(self, rhs: DateTime<T>) -> Self::Output {
		rhs + Duration::from(self)
	}
}

impl<T> Add<DurationFlex> for DateTime<T>
where
	T: TimeZone,
{
	type Output = DateTime<T>;

	fn add(self, rhs: DurationFlex) -> Self::Output {
		self + Duration::from(rhs)
	}
}

impl TryFrom<&str> for DurationFlex {
	type Error = DurationFlexError;

	fn try_from(value: &str) -> Result<Self, Self::Error> {
		let captures = REGEX.captures(value).ok_or(DurationFlexError::InvalidFormat)?;

		let weeks = Duration::try_weeks(captures.name("weeks").map_or(0i64, Self::de_component))
			.ok_or(DurationFlexError::OutOfRange)?;
		let days = Duration::try_days(captures.name("days").map_or(0i64, Self::de_component))
			.ok_or(DurationFlexError::OutOfRange)?;
		let hours = Duration::try_hours(captures.name("hours").map_or(0i64, Self::de_component))
			.ok_or(DurationFlexError::OutOfRange)?;
		let minutes = Duration::try_minutes(captures.name("minutes").map_or(0i64, Self::de_component))
			.ok_or(DurationFlexError::OutOfRange)?;
		let seconds = Duration::try_seconds(captures.name("seconds").map_or(0i64, Self::de_component))
			.ok_or(DurationFlexError::OutOfRange)?;

		let duration = weeks + days + hours + minutes + seconds;

		Ok(DurationFlex { secs: duration.num_seconds(), nanos: 0i32 })
	}
}

impl From<String> for DurationFlex {
	fn from(value: String) -> Self {
		DurationFlex::try_from(value.as_str()).unwrap()
	}
}

impl From<Duration> for DurationFlex {
	fn from(value: Duration) -> Self {
		DurationFlex { secs: value.num_seconds(), nanos: 0i32 }
	}
}

impl From<DurationFlex> for Duration {
	fn from(value: DurationFlex) -> Self {
		Duration::try_seconds(value.secs()).unwrap() + Duration::nanoseconds(value.nanos() as i64)
	}
}

impl From<time::Duration> for DurationFlex {
	fn from(value: time::Duration) -> Self {
		DurationFlex { secs: value.as_secs() as i64, nanos: 0i32 }
	}
}

impl From<DurationFlex> for time::Duration {
	fn from(value: DurationFlex) -> Self {
		time::Duration::from_secs(value.secs as u64).add(time::Duration::from_nanos(value.nanos as u64))
	}
}

impl Display for DurationFlex {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut secs = self.secs;

		Self::ser_component(&mut secs, "w", SECS_PER_WEEK, f)?;
		Self::ser_component(&mut secs, "d", SECS_PER_DAY, f)?;
		Self::ser_component(&mut secs, "h", SECS_PER_HOUR, f)?;
		Self::ser_component(&mut secs, "m", SECS_PER_MINUTES, f)?;
		Self::ser_component(&mut secs, "s", 1, f)
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for DurationFlex {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		static REGEX_MSG: &str =
			"a String with the format weeks (w), days (d), hours (h), minutes (m) and/or seconds (s), in order";

		struct DurationFlexVisitor;

		impl<'de> Visitor<'de> for DurationFlexVisitor {
			type Value = DurationFlex;

			fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
				formatter.write_str(REGEX_MSG)
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: Error,
			{
				match DurationFlex::try_from(v) {
					Ok(value) => Ok(value),
					Err(DurationFlexError::InvalidFormat) => Err(Error::invalid_value(Unexpected::Str(v), &self)),
					Err(DurationFlexError::OutOfRange) => Err(Error::invalid_value(Unexpected::Str(v), &self)),
				}
			}

			fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
			where
				E: Error,
			{
				self.visit_str(v)
			}

			fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
			where
				E: Error,
			{
				match DurationFlex::try_from(v.as_str()) {
					Ok(value) => Ok(value),
					Err(DurationFlexError::InvalidFormat) => {
						Err(Error::invalid_value(Unexpected::Str(v.as_str()), &self))
					},
					Err(DurationFlexError::OutOfRange) => Err(Error::invalid_value(Unexpected::Str(v.as_str()), &self)),
				}
			}
		}

		deserializer.deserialize_string(DurationFlexVisitor)
	}
}

#[cfg(feature = "serde")]
impl Serialize for DurationFlex {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(format!("{}", self).as_str())
	}
}

#[cfg(feature = "clap")]
impl From<OsStr> for DurationFlex {
	fn from(value: OsStr) -> Self {
		DurationFlex::try_from(value.to_str().unwrap()).unwrap()
	}
}

#[cfg(feature = "clap")]
impl From<DurationFlex> for OsStr {
	fn from(value: DurationFlex) -> Self {
		format!("{}", value).into()
	}
}

impl FromStr for DurationFlex {
	type Err = DurationFlexError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		DurationFlex::try_from(s)
	}
}

#[cfg(test)]
mod test {

	use serde::{Deserialize, Serialize};
	use serde_test::{assert_de_tokens, assert_ser_tokens, Token};

	use super::*;

	#[test]
	fn de_string() {
		let value = DurationFlex::try_from("1w2d").unwrap();
		assert_eq!(value.secs(), 9 * SECS_PER_DAY);
		assert_eq!(value.nanos(), 0);

		let value = DurationFlex::try_from("1w2d3h4m5s").unwrap();
		assert_eq!(value.secs(), 9 * SECS_PER_DAY + 3 * SECS_PER_HOUR + 4 * SECS_PER_MINUTES + 5);
		assert_eq!(value.nanos(), 0);

		let value = DurationFlex::try_from("5s").unwrap();
		assert_eq!(value.secs(), 5);
		assert_eq!(value.nanos(), 0);

		let value = DurationFlex::try_from("5s5d");
		assert!(value.is_err());
	}

	#[test]
	fn ser_string() {
		let value = DurationFlex::try_from("1w2d").unwrap().to_string();
		assert_eq!(value, "1w2d");

		let value = DurationFlex::try_from("1w2d3h4m5s").unwrap().to_string();
		assert_eq!(value, "1w2d3h4m5s");

		let value = DurationFlex::try_from("5s").unwrap().to_string();
		assert_eq!(value, "5s");

		let value = DurationFlex::try_from("1w8d3h4m5s").unwrap().to_string();
		assert_eq!(value, "2w1d3h4m5s");

		let value = DurationFlex::try_from("1w8d3h4m3605s").unwrap().to_string();
		assert_eq!(value, "2w1d4h4m5s");
	}

	#[test]
	fn deserialize_nums() {
		let value = DurationFlex::try_from("1w2d").unwrap();
		assert_de_tokens(&value, &[Token::Str("1w2d")]);

		let value = DurationFlex::try_from("1w2d3h4m5s").unwrap();
		assert_de_tokens(&value, &[Token::Str("1w2d3h4m5s")]);

		let value = DurationFlex::try_from("5s").unwrap();
		assert_de_tokens(&value, &[Token::Str("5s")]);

		let value = DurationFlex::try_from("1w8d3h4m5s").unwrap();
		assert_de_tokens(&value, &[Token::Str("2w1d3h4m5s")]);

		let value = DurationFlex::try_from("1w8d3h4m3605s").unwrap();
		assert_de_tokens(&value, &[Token::Str("2w1d4h4m5s")]);
	}

	#[test]
	fn serialize() {
		let value = DurationFlex::try_from("1w2d").unwrap();
		assert_ser_tokens(&value, &[Token::Str("1w2d")]);

		let value = DurationFlex::try_from("1w2d3h4m5s").unwrap();
		assert_ser_tokens(&value, &[Token::Str("1w2d3h4m5s")]);

		let value = DurationFlex::try_from("5s").unwrap();
		assert_ser_tokens(&value, &[Token::Str("5s")]);

		let value = DurationFlex::try_from("1w8d3h4m5s").unwrap();
		assert_ser_tokens(&value, &[Token::Str("2w1d3h4m5s")]);

		let value = DurationFlex::try_from("1w8d3h4m3605s").unwrap();
		assert_ser_tokens(&value, &[Token::Str("2w1d4h4m5s")]);
	}

	#[test]
	fn in_struct() {
		#[derive(Serialize, Deserialize)]
		struct SomeStruct {
			duration: DurationFlex,
		}

		let value = SomeStruct { duration: Duration::try_weeks(1).unwrap().into() };

		assert_ser_tokens(
			&value,
			&[Token::Struct { name: "SomeStruct", len: 1 }, Token::Str("duration"), Token::Str("1w"), Token::StructEnd],
		);
	}
}
