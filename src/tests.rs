extern crate std;

use std::prelude::rust_2024::*;

// todo: more tests

fn extract<'a, T, E: core::fmt::Debug>(
	command: crate::Command<T, E>,
	config: crate::Config,
	argument: &[&'a str],
	layer: T,
) -> (Result<crate::Success<T>, crate::Error<'a, E>>, String) {
	let mut string = String::new();
	let output = crate::Output::new()
		.writer_fmt(&mut string);
	let value = crate::parse_str(output, config, command, argument.iter().copied(), layer);
	(value, string)
}

#[test]
fn test_default() {
	const COMMAND: crate::Command<(), ()> = crate::Command::new();
	const CONFIG: crate::Config = crate::Config::new();
	let result = extract(COMMAND, CONFIG, &[], ());
	assert_eq!(result, (Ok(crate::Success::Layer(())), String::new()));
}

#[test]
fn test_help_default() {
	const COMMAND: crate::Command<(), ()> = crate::Command::new()
		.argument(&[
			crate::Argument::new()
				.action_help(),
		]);
	const CONFIG: crate::Config = crate::Config::new();
	let result = extract(COMMAND, CONFIG, &[""], ());

	let check = "\
		usage:[<unknown>]\n\
		options:\n\
	";

	assert_eq!(result, (Ok(crate::Success::Help), check.to_string()));
}

#[test]
fn test_help_one_named_argument_no_description() {
	const COMMAND: crate::Command<(), ()> = crate::Command::new()
		.argument(&[
			crate::Argument::new()
				.named(&["help"], &['h'])
				.action_help(),
		]);
	const CONFIG: crate::Config = crate::Config::new();
	let result = extract(COMMAND, CONFIG, &["--help"], ());

	let check = "\
usage: <options>
options:
  -h, --help
";

	assert_eq!(result, (Ok(crate::Success::Help), check.to_string()));
}

#[test]
fn test_help_one_named_argument_with_description() {
	const COMMAND: crate::Command<(), ()> = crate::Command::new()
		.argument(&[
			crate::Argument::new()
				.named(&["help"], &['h'])
				.help("this displays help information for anyone that may need it, someday, someday. sorry I need another sentence to test wrapping.")
				.action_help(),
		]);
	const CONFIG: crate::Config = crate::Config::new();
	let result = extract(COMMAND, CONFIG, &["--help"], ());

	let check = "\
usage: <options>
options:
  -h, --help
         this displays help information for anyone that may need it,
         someday, someday. sorry I need another sentence to test
         wrapping.
";

	assert_eq!(result, (Ok(crate::Success::Help), check.to_string()));
}

#[test]
fn test_positional() {
	const COMMAND: crate::Command<Vec<String>, ()> = crate::Command::new()
		.argument(&[
			crate::Argument::new()
				.positional("a")
				.action_layer(|mut input, next| {
					let a = next().map(String::from).unwrap();
					input.push(a);
					assert_eq!(next(), None);
					Ok(input)
				}),
			crate::Argument::new()
				.positional("b")
				.action_layer(|mut input, next| {
					let a = next().map(String::from).unwrap();
					input.push(a);
					assert_eq!(next(), None);
					Ok(input)
				}),
		]);

	const CONFIG: crate::Config = crate::Config::new();

	let result = extract(COMMAND, CONFIG, &[], Vec::new());
	assert_eq!(result, (Ok(crate::Success::Layer(vec![])), String::new()));

	let result = extract(COMMAND, CONFIG, &["meow"], Vec::new());
	assert_eq!(result, (Ok(crate::Success::Layer(vec!["meow".to_string()])), String::new()));

	let result = extract(COMMAND, CONFIG, &["meow-x", "meow-y"], Vec::new());
	assert_eq!(result, (Ok(crate::Success::Layer(vec!["meow-x".to_string(), "meow-y".to_string()])), String::new()));

	let result = extract(COMMAND, CONFIG, &["meow-x", "meow-y", "uh oh"], Vec::new());
	assert_eq!(result, (Err(crate::Error::BadPositional("uh oh")), String::new()));
}

#[test]
fn test_positional_truncation() {
	const COMMAND: crate::Command<Vec<String>, ()> = crate::Command::new();

	const CONFIG: crate::Config = crate::Config::new();

	let check = "abcdefghjiklmnopqrstuvwxyz0123456789!@#$%^&*()";

	let result = extract(COMMAND, CONFIG, &[check], Vec::new());
	assert_eq!(result, (Err(crate::Error::BadPositional(&check[0..31])), String::new()));
}

#[test]
fn test_name_splitting() {
	const COMMAND: crate::Command<(Option<String>, Option<String>), ()> = crate::Command::new()
		.argument(&[
			crate::Argument::new()
				.named(&["enabled"], &['x'])
				.splitting(true)
				.action_layer(|_, next| {
					let a = next().map(String::from);
					let b = next().map(String::from);
					Ok((a, b))
				}),
			crate::Argument::new()
				.named(&["disabled"], &['y'])
				.splitting(false)
				.action_layer(|_, next| {
					let a = next().map(String::from);
					let b = next().map(String::from);
					Ok((a, b))
				}),
		]);

	const CONFIG: crate::Config = crate::Config::new()
		.splitting("=");

	let result = extract(COMMAND, CONFIG, &["--enabled=meow"], (None, None));
	assert_eq!(result, (Ok(crate::Success::Layer((Some("meow".to_string()), None))), String::new()));

	let result = extract(COMMAND, CONFIG, &["--enabled==meow"], (None, None));
	assert_eq!(result, (Ok(crate::Success::Layer((Some("=meow".to_string()), None))), String::new()));

	let result = extract(COMMAND, CONFIG, &["--enabled", "meow"], (None, None));
	assert_eq!(result, (Ok(crate::Success::Layer((Some("meow".to_string()), None))), String::new()));

	let result = extract(COMMAND, CONFIG, &["--enabled=meow-x", "meow-y"], (None, None));
	assert_eq!(result, (Ok(crate::Success::Layer((Some("meow-x".to_string()), Some("meow-y".to_string())))), String::new()));

	let result = extract(COMMAND, CONFIG, &["-x=meow"], (None, None));
	assert_eq!(result, (Err(crate::Error::BadShort('=')), String::new()));

	let result = extract(COMMAND, CONFIG, &["--disabled=meow"], (None, None));
	assert_eq!(result, (Err(crate::Error::BadLong("disabled=meow")), String::new()));
}
