#![allow(clippy::tabs_in_doc_comments)]
//! # Duration Flex
//!
//! Helper to make it easier to specify durations. Specially useful in configuration files.
//! - Basic interoperability with [`chrono::DateTime`], allowing it to be added/subbed from it.
//! - Can be built from [`chrono::Duration`].
//! - Can be built from [`std::time::Duration`].
//!
//! **Example:**
//! - 1 hour and 23 minutes: `1h23m`
//! - 1 week, 6 days, 23 hours, 49 minutes and 50 seconds: `1w6d23h49m59s`
//! - 1 year, 2 weeks and 3 days: `1y2w3d`
//!
//! **Supported Time Units**
//! - Years: `1y` (1 year, equivalent to 365 days).
//! - Weeks: `2w` (2 weeks).
//! - Days: `3d` (3 days).
//! - Hours: `15h` (15 hours).
//! - Minutes: `5m` (5 minutes).
//! - Seconds: `30s` (30 seconds).
//!
//! > **Note:** Months are not supported because they vary in the amount of days (28 to 31 days).
//! > It is best to specify the desired duration in days instead (e.g. `30d`).
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
//! - `utoipa`: enable support for the [`utoipa`] crate, allowing it to be used with the `ToSchema` derivation.
//! - `validator`: enable support for the [`validator`] crate, allowing it to be used with the `range` validator.
//!
//! ### Validator Example:
//!
//! You can specify the range using the fully qualified type (extended version):
//! ```
//! # #[cfg(feature = "validator")]
//! # {
//! use duration_flex::DurationFlex;
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct Config {
//! 	#[validate(range(
//! 		min = "DurationFlex::try_from(\"1h\").unwrap()",
//! 		max = "DurationFlex::try_from(\"2h\").unwrap()"
//! 	))]
//! 	timeout: DurationFlex,
//! }
//! # }
//! ```
//!
//! Or using string literals (string version). Note the escaped inner quotes, which are required
//! because the macro parses the arguments as Rust expressions:
//! ```
//! # #[cfg(feature = "validator")]
//! # {
//! use duration_flex::DurationFlex;
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct Config {
//! 	#[validate(range(min = "\"1h\"", max = "\"2h\""))]
//! 	timeout: DurationFlex,
//! }
//! # }
//! ```
//!
//! Or using numbers (number version), which represent the amount of seconds:
//! ```
//! # #[cfg(feature = "validator")]
//! # {
//! use duration_flex::DurationFlex;
//! use validator::Validate;
//!
//! #[derive(Validate)]
//! struct Config {
//! 	#[validate(range(min = 3600, max = 7200))]
//! 	timeout: DurationFlex,
//! }
//! # }
//! ```

use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, Div, Mul, Sub};
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
const SECS_PER_YEAR: i64 = 365 * SECS_PER_DAY;

/// Errors returned by the different methods.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum DurationFlexError {
	/// String format is not valid, e.g. `1x` (`x` is not supported).
	InvalidFormat,

	/// Value is out of range.
	OutOfRange,
}

impl Display for DurationFlexError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			DurationFlexError::InvalidFormat => write!(f, "invalid duration format"),
			DurationFlexError::OutOfRange => write!(f, "duration value is out of range"),
		}
	}
}

impl std::error::Error for DurationFlexError {}

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
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "utoipa", schema(as = String, example = "1h30m"))]
pub struct DurationFlex {
	secs: i64,
	nanos: i32,
}

#[cfg(feature = "validator")]
impl validator::ValidateRange<DurationFlex> for DurationFlex {
	fn greater_than(&self, max: DurationFlex) -> Option<bool> {
		Some(self > &max)
	}

	fn less_than(&self, min: DurationFlex) -> Option<bool> {
		Some(self < &min)
	}
}

#[cfg(feature = "validator")]
impl validator::ValidateRange<&str> for DurationFlex {
	fn greater_than(&self, max: &str) -> Option<bool> {
		let max = DurationFlex::try_from(max).expect("invalid duration string in validator bounds");
		Some(self > &max)
	}

	fn less_than(&self, min: &str) -> Option<bool> {
		let min = DurationFlex::try_from(min).expect("invalid duration string in validator bounds");
		Some(self < &min)
	}
}

#[cfg(feature = "validator")]
impl validator::ValidateRange<i64> for DurationFlex {
	fn greater_than(&self, max: i64) -> Option<bool> {
		Some(self.secs > max)
	}

	fn less_than(&self, min: i64) -> Option<bool> {
		Some(self.secs < min)
	}
}

static REGEX_STR: &str = r"^((?P<years>\d+)y)?((?P<weeks>\d+)w)?((?P<days>\d+)d)?((?P<hours>\d+)h)?((?P<minutes>\d+)m)?((?P<seconds>\d+)s)?$";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(REGEX_STR).unwrap());

impl DurationFlex {
	/// A duration of zero seconds.
	pub const ZERO: DurationFlex = DurationFlex { secs: 0, nanos: 0 };

	/// Creates a new `DurationFlex` from seconds and nanoseconds.
	pub const fn new(secs: i64, nanos: i32) -> Self {
		DurationFlex { secs, nanos }
	}

	/// Creates a new `DurationFlex` from a whole number of seconds.
	pub const fn from_secs(secs: i64) -> Self {
		DurationFlex { secs, nanos: 0 }
	}

	/// Creates a new `DurationFlex` from a whole number of milliseconds.
	pub const fn from_millis(millis: i64) -> Self {
		let secs = millis / 1000;
		let extra_millis = millis % 1000;
		DurationFlex { secs, nanos: (extra_millis * 1_000_000) as i32 }
	}

	/// Creates a new `DurationFlex` from a whole number of minutes.
	pub const fn from_minutes(minutes: i64) -> Self {
		DurationFlex { secs: minutes * SECS_PER_MINUTES, nanos: 0 }
	}

	/// Creates a new `DurationFlex` from a whole number of hours.
	pub const fn from_hours(hours: i64) -> Self {
		DurationFlex { secs: hours * SECS_PER_HOUR, nanos: 0 }
	}

	/// Creates a new `DurationFlex` from a whole number of days.
	pub const fn from_days(days: i64) -> Self {
		DurationFlex { secs: days * SECS_PER_DAY, nanos: 0 }
	}

	/// Creates a new `DurationFlex` from a whole number of weeks.
	pub const fn from_weeks(weeks: i64) -> Self {
		DurationFlex { secs: weeks * SECS_PER_WEEK, nanos: 0 }
	}

	/// Creates a new `DurationFlex` from a whole number of years (365 days each).
	pub const fn from_years(years: i64) -> Self {
		DurationFlex { secs: years * SECS_PER_YEAR, nanos: 0 }
	}

	/// Returns true if the duration is zero.
	pub const fn is_zero(&self) -> bool {
		self.secs == 0 && self.nanos == 0
	}

	/// Returns true if the duration is positive (> 0).
	pub const fn is_positive(&self) -> bool {
		self.secs > 0 || (self.secs == 0 && self.nanos > 0)
	}

	/// Returns true if the duration is negative (< 0).
	pub const fn is_negative(&self) -> bool {
		self.secs < 0 || (self.secs == 0 && self.nanos < 0)
	}

	/// Converts this duration into a [`std::time::Duration`] if non-negative.
	pub fn to_std(&self) -> Option<time::Duration> {
		if self.secs < 0 || (self.secs == 0 && self.nanos < 0) {
			None
		} else {
			Some(time::Duration::new(self.secs as u64, self.nanos as u32))
		}
	}

	/// Converts this duration into a [`chrono::Duration`].
	pub fn to_chrono(&self) -> Duration {
		Duration::from(*self)
	}

	/// Formats the duration in a human-readable English string (e.g. `"1 year, 2 weeks, 3 days"`).
	///
	/// If the duration is zero, returns `"0 seconds"`.
	pub fn format_human(&self) -> String {
		if self.is_zero() {
			return "0 seconds".to_string();
		}

		let mut secs = self.secs;
		let mut parts = Vec::new();

		let units = [
			(SECS_PER_YEAR, "year", "years"),
			(SECS_PER_WEEK, "week", "weeks"),
			(SECS_PER_DAY, "day", "days"),
			(SECS_PER_HOUR, "hour", "hours"),
			(SECS_PER_MINUTES, "minute", "minutes"),
			(1, "second", "seconds"),
		];

		for (unit_secs, singular, plural) in units {
			let count = secs / unit_secs;
			secs -= count * unit_secs;
			if count > 0 {
				if count == 1 {
					parts.push(format!("1 {singular}"));
				} else {
					parts.push(format!("{count} {plural}"));
				}
			}
		}

		parts.join(", ")
	}

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

impl Sub<DurationFlex> for DurationFlex {
	type Output = DurationFlex;

	fn sub(self, rhs: DurationFlex) -> Self::Output {
		DurationFlex { secs: self.secs - rhs.secs, nanos: self.nanos - rhs.nanos }
	}
}

impl Add<DurationFlex> for DurationFlex {
	type Output = DurationFlex;

	fn add(self, rhs: DurationFlex) -> Self::Output {
		DurationFlex { secs: self.secs + rhs.secs, nanos: self.nanos + rhs.nanos }
	}
}

impl Mul<u32> for DurationFlex {
	type Output = DurationFlex;

	fn mul(self, rhs: u32) -> Self::Output {
		DurationFlex { secs: self.secs * rhs as i64, nanos: self.nanos * rhs as i32 }
	}
}

impl Mul<i64> for DurationFlex {
	type Output = DurationFlex;

	fn mul(self, rhs: i64) -> Self::Output {
		DurationFlex { secs: self.secs * rhs, nanos: (self.nanos as i64 * rhs) as i32 }
	}
}

impl Div<u32> for DurationFlex {
	type Output = DurationFlex;

	fn div(self, rhs: u32) -> Self::Output {
		DurationFlex { secs: self.secs / rhs as i64, nanos: self.nanos / rhs as i32 }
	}
}

impl Div<i64> for DurationFlex {
	type Output = DurationFlex;

	fn div(self, rhs: i64) -> Self::Output {
		DurationFlex { secs: self.secs / rhs, nanos: (self.nanos as i64 / rhs) as i32 }
	}
}

impl Sum for DurationFlex {
	fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
		iter.fold(DurationFlex::default(), |acc, x| acc + x)
	}
}

impl<'a> Sum<&'a DurationFlex> for DurationFlex {
	fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
		iter.fold(DurationFlex::default(), |acc, x| acc + *x)
	}
}

impl<T> Sub<DurationFlex> for DateTime<T>
where
	T: TimeZone,
{
	type Output = DateTime<T>;

	fn sub(self, rhs: DurationFlex) -> Self::Output {
		self - Duration::from(rhs)
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

		let years = Duration::try_days(captures.name("years").map_or(0i64, Self::de_component) * 365)
			.ok_or(DurationFlexError::OutOfRange)?;
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

		let duration = years + weeks + days + hours + minutes + seconds;

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

		Self::ser_component(&mut secs, "y", SECS_PER_YEAR, f)?;
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
		static REGEX_MSG: &str = "a String with the format years (y), weeks (w), days (d), hours (h), minutes (m) \
		                          and/or seconds (s), in order";

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
		let value = DurationFlex::try_from("1y").unwrap();
		assert_eq!(value.secs(), SECS_PER_YEAR);
		assert_eq!(value.nanos(), 0);

		let value = DurationFlex::try_from("1y2w3d4h5m6s").unwrap();
		assert_eq!(
			value.secs(),
			SECS_PER_YEAR + 2 * SECS_PER_WEEK + 3 * SECS_PER_DAY + 4 * SECS_PER_HOUR + 5 * SECS_PER_MINUTES + 6
		);
		assert_eq!(value.nanos(), 0);

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

		let value = DurationFlex::try_from("1w1y");
		assert!(value.is_err());
	}

	#[test]
	fn ser_string() {
		let value = DurationFlex::try_from("1y").unwrap().to_string();
		assert_eq!(value, "1y");

		let value = DurationFlex::try_from("1y2w3d4h5m6s").unwrap().to_string();
		assert_eq!(value, "1y2w3d4h5m6s");

		let value = DurationFlex::try_from("365d").unwrap().to_string();
		assert_eq!(value, "1y");

		let value = DurationFlex::try_from("372d").unwrap().to_string();
		assert_eq!(value, "1y1w");

		let value = DurationFlex::try_from("53w").unwrap().to_string();
		assert_eq!(value, "1y6d");

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
		let value = DurationFlex::try_from("1y").unwrap();
		assert_de_tokens(&value, &[Token::Str("1y")]);

		let value = DurationFlex::try_from("1y2w3d4h5m6s").unwrap();
		assert_de_tokens(&value, &[Token::Str("1y2w3d4h5m6s")]);

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
		let value = DurationFlex::try_from("1y").unwrap();
		assert_ser_tokens(&value, &[Token::Str("1y")]);

		let value = DurationFlex::try_from("1y2w3d4h5m6s").unwrap();
		assert_ser_tokens(&value, &[Token::Str("1y2w3d4h5m6s")]);

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

		let value_year = SomeStruct { duration: DurationFlex::try_from("1y").unwrap() };

		assert_ser_tokens(
			&value_year,
			&[Token::Struct { name: "SomeStruct", len: 1 }, Token::Str("duration"), Token::Str("1y"), Token::StructEnd],
		);
	}

	#[cfg(feature = "validator")]
	#[test]
	fn validator() {
		use validator::Validate;

		#[derive(Validate)]
		struct SomeStruct {
			#[validate(range(
				min = "DurationFlex::try_from(\"1h\").unwrap()",
				max = "DurationFlex::try_from(\"2h\").unwrap()"
			))]
			duration: DurationFlex,
		}

		let value = SomeStruct { duration: DurationFlex::try_from("1h30m").unwrap() };
		assert!(value.validate().is_ok());

		let value = SomeStruct { duration: DurationFlex::try_from("30m").unwrap() };
		assert!(value.validate().is_err());

		let value = SomeStruct { duration: DurationFlex::try_from("2h30m").unwrap() };
		assert!(value.validate().is_err());

		#[derive(Validate)]
		struct YearStruct {
			#[validate(range(
				min = "DurationFlex::try_from(\"1y\").unwrap()",
				max = "DurationFlex::try_from(\"2y\").unwrap()"
			))]
			duration: DurationFlex,
		}

		let value = YearStruct { duration: DurationFlex::try_from("1y6w").unwrap() };
		assert!(value.validate().is_ok());

		let value = YearStruct { duration: DurationFlex::try_from("300d").unwrap() };
		assert!(value.validate().is_err());

		let value = YearStruct { duration: DurationFlex::try_from("2y1d").unwrap() };
		assert!(value.validate().is_err());
	}

	#[cfg(feature = "validator")]
	#[test]
	fn validator_str() {
		use validator::Validate;

		#[derive(Validate)]
		struct SomeStruct {
			#[validate(range(min = "\"1h\"", max = "\"2h\""))]
			duration: DurationFlex,
		}

		let value = SomeStruct { duration: DurationFlex::try_from("1h30m").unwrap() };
		assert!(value.validate().is_ok());

		let value = SomeStruct { duration: DurationFlex::try_from("30m").unwrap() };
		assert!(value.validate().is_err());

		let value = SomeStruct { duration: DurationFlex::try_from("2h30m").unwrap() };
		assert!(value.validate().is_err());

		#[derive(Validate)]
		struct YearStruct {
			#[validate(range(min = "\"1y\"", max = "\"2y\""))]
			duration: DurationFlex,
		}

		let value = YearStruct { duration: DurationFlex::try_from("1y6w").unwrap() };
		assert!(value.validate().is_ok());

		let value = YearStruct { duration: DurationFlex::try_from("300d").unwrap() };
		assert!(value.validate().is_err());

		let value = YearStruct { duration: DurationFlex::try_from("2y1d").unwrap() };
		assert!(value.validate().is_err());
	}

	#[cfg(feature = "validator")]
	#[test]
	fn validator_int() {
		use validator::Validate;

		#[derive(Validate)]
		struct SomeStruct {
			#[validate(range(min = 3600, max = 7200))]
			duration: DurationFlex,
		}

		let value = SomeStruct { duration: DurationFlex::try_from("1h30m").unwrap() };
		assert!(value.validate().is_ok());

		let value = SomeStruct { duration: DurationFlex::try_from("30m").unwrap() };
		assert!(value.validate().is_err());

		let value = SomeStruct { duration: DurationFlex::try_from("2h30m").unwrap() };
		assert!(value.validate().is_err());
	}

	#[test]
	fn default_and_hash() {
		use std::collections::HashSet;

		let default_val = DurationFlex::default();
		assert_eq!(default_val.secs(), 0);
		assert_eq!(default_val.nanos(), 0);

		let mut set = HashSet::new();
		set.insert(DurationFlex::try_from("1h").unwrap());
		set.insert(DurationFlex::try_from("60m").unwrap());
		assert_eq!(set.len(), 1);

		let mut err_set = HashSet::new();
		err_set.insert(DurationFlexError::InvalidFormat);
		err_set.insert(DurationFlexError::OutOfRange);
		assert_eq!(err_set.len(), 2);
	}

	#[test]
	fn error_traits() {
		use std::error::Error;

		let err = DurationFlexError::InvalidFormat;
		assert_eq!(err.to_string(), "invalid duration format");
		let err_source: &dyn Error = &err;
		assert!(err_source.source().is_none());

		let err_oor = DurationFlexError::OutOfRange;
		assert_eq!(err_oor.to_string(), "duration value is out of range");
	}

	#[test]
	fn arithmetic_operations() {
		let a = DurationFlex::try_from("1h").unwrap();
		let b = DurationFlex::try_from("30m").unwrap();

		assert_eq!((a + b).secs(), 90 * 60);
		assert_eq!((a - b).secs(), 30 * 60);
		assert_eq!((b * 2u32).secs(), 3600);
		assert_eq!((b * 3i64).secs(), 5400);
		assert_eq!((a / 2u32).secs(), 1800);
		assert_eq!((a / 4i64).secs(), 900);

		let list = vec![
			DurationFlex::try_from("1h").unwrap(),
			DurationFlex::try_from("30m").unwrap(),
			DurationFlex::try_from("15m").unwrap(),
		];
		let total: DurationFlex = list.iter().sum();
		assert_eq!(total.secs(), 105 * 60);

		let total_owned: DurationFlex = list.into_iter().sum();
		assert_eq!(total_owned.secs(), 105 * 60);
	}

	#[test]
	fn datetime_subtraction() {
		use chrono::Utc;

		let now = Utc::now();
		let duration = DurationFlex::try_from("1h").unwrap();
		let earlier = now - duration;
		assert_eq!(earlier + duration, now);
	}

	#[test]
	fn constructors_and_inspectors() {
		assert_eq!(DurationFlex::ZERO, DurationFlex::new(0, 0));
		assert!(DurationFlex::ZERO.is_zero());
		assert!(!DurationFlex::ZERO.is_positive());
		assert!(!DurationFlex::ZERO.is_negative());

		let s = DurationFlex::from_secs(10);
		assert_eq!(s.secs(), 10);
		assert_eq!(s.nanos(), 0);
		assert!(s.is_positive());
		assert!(!s.is_negative());

		let ms = DurationFlex::from_millis(1500);
		assert_eq!(ms.secs(), 1);
		assert_eq!(ms.nanos(), 500_000_000);

		let m = DurationFlex::from_minutes(2);
		assert_eq!(m.secs(), 120);

		let h = DurationFlex::from_hours(3);
		assert_eq!(h.secs(), 3 * 3600);

		let d = DurationFlex::from_days(4);
		assert_eq!(d.secs(), 4 * 86400);

		let w = DurationFlex::from_weeks(2);
		assert_eq!(w.secs(), 14 * 86400);

		let y = DurationFlex::from_years(1);
		assert_eq!(y.secs(), 365 * 86400);

		let neg = DurationFlex::new(-5, 0);
		assert!(neg.is_negative());
		assert!(!neg.is_positive());
		assert_eq!(neg.to_std(), None);

		let pos = DurationFlex::from_secs(5);
		assert_eq!(pos.to_std(), Some(time::Duration::from_secs(5)));
		assert_eq!(pos.to_chrono(), Duration::try_seconds(5).unwrap());
	}

	#[test]
	fn test_format_human() {
		assert_eq!(DurationFlex::ZERO.format_human(), "0 seconds");
		assert_eq!(DurationFlex::from_secs(1).format_human(), "1 second");
		assert_eq!(DurationFlex::from_secs(30).format_human(), "30 seconds");
		assert_eq!(DurationFlex::from_minutes(1).format_human(), "1 minute");
		assert_eq!(DurationFlex::from_minutes(5).format_human(), "5 minutes");
		assert_eq!(DurationFlex::from_hours(1).format_human(), "1 hour");
		assert_eq!(DurationFlex::from_hours(2).format_human(), "2 hours");
		assert_eq!(DurationFlex::from_days(1).format_human(), "1 day");
		assert_eq!(DurationFlex::from_days(3).format_human(), "3 days");
		assert_eq!(DurationFlex::from_weeks(1).format_human(), "1 week");
		assert_eq!(DurationFlex::from_weeks(2).format_human(), "2 weeks");
		assert_eq!(DurationFlex::from_years(1).format_human(), "1 year");
		assert_eq!(DurationFlex::from_years(2).format_human(), "2 years");

		let complex = DurationFlex::try_from("1y2w3d4h5m6s").unwrap();
		assert_eq!(complex.format_human(), "1 year, 2 weeks, 3 days, 4 hours, 5 minutes, 6 seconds");
	}
}
