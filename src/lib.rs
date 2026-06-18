#![doc = include_str!("../readme.md")]
//! ## usage
//!
//! there are four main pieces:
//!
//! - [`crate::Command`] is the main piece, configuring what arguments and
//! subcommands the cli will have.
//! - [`crate::Config`] configures options related to global parsing logic and output.
//! - [`crate::Output`] allows you to set where the output of the cli goes (for
//! example, into a `String`, or stdio).
//! - [`crate::parse()`], finally, is what parses and handles arguments.
//!
//! ```
//! #[derive(Debug)]
//! struct Input(Option<u32>);
//!
//! #[derive(Debug)]
//! struct Error(String);
//!
//! const command: purcarg::Command<Input, Error> = purcarg::Command::new()
//!     .name(&["example"])
//!     .description("provides examples as to how to use purcarg")
//!     .argument(&[
//!         purcarg::Argument::new()
//!             .positional("parse")
//!             .help("text to parse into a number then return as text")
//!             .action_layer(|mut input: Input, next| {
//!                 let number: u32 = next()
//!                     .and_then(|x| x.parse::<u32>().ok())
//!                     .ok_or(Error("invalid input".to_string()))?;
//!                 input.0 = Some(number);
//!                 Ok(input)
//!             })
//!             .required(true),
//!         purcarg::Argument::new()
//!             .named(&["help"], &['h'])
//!             .help("print this help message then exit")
//!             .action_help(),
//!     ]);
//!
//! const config: purcarg::Config = purcarg::Config::new();
//!
//! let output = purcarg::Output::new()
//!     .writer_stdio(); // write to stdio
//!
//! let arguments = ["10"];
//!
//! let input = purcarg::parse(output, config, command, arguments, Input(None)).unwrap();
//!
//! assert_eq!(input.0, Some(10));
//! ```

#![no_std]

#![warn(missing_docs)]
#![warn(clippy::cargo)]

#[cfg(feature = "std")]
extern crate std;

mod types;
mod parse;

#[cfg(test)]
mod tests;

pub use types::*;
pub use parse::*;
