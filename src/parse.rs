#[allow(clippy::wildcard_imports, reason = "this is self contained")]
use crate::types::*;

/// parse command line options.
///
/// ```
/// // configures how output should be written - ie, into a string, or stdio.
/// let output = purcarg::Output::new();
///
/// // configures how output should be formatted globally.
/// let config = purcarg::Config::new();
///
/// // configures routing (what options or subcommands to expect) and other metadata.
/// const command: purcarg::Command<(), ()> = purcarg::Command::new();
///
/// purcarg::parse_str(output, config, command, [], ());
/// ```
///
/// `argument` is an iterator of `&str`, each representing a single option or
/// input with no trailing whitespace (`-abc --xyz 123` ->
/// `"-abc" "--xyz" "123"`).
///
/// it being a simple, strictly ordered iterator, plus this parse having
/// no allocation, places certain restrictions in how arguments can be parsed,
/// and may lead to surprising behaviour. the rules used will be described
/// here.
///
/// - the parser first starts in "named option mode". for each argument, it
///   checks if it begins with either long style `"--"` or short style `"-"`.
///     - in the case of long style, the `"--"` is first trimmed off to get the
///       name. if [`field@crate::Config::name_splitting`] is enabled, then the
///       name is first split into a name and value. then name is then matched
///       against `command`'s arguments that are [`crate::ArgumentForm::Named`]
///       to run its respective `Action`.
///     - in the case of short style, the `"-"` is first trimmed off. then,
///       each character is matched against `command`'s arguments that are
///       [`crate::ArgumentForm::Named`] to run each respective `Action`
///       (eg: `-abc`). notably, if any of these `Action`s are
///       [`crate::Action::Layer`], and the layer's passed in `next()` function
///       is called, only the last character (so, the `c` in `-abc`) will
///       recieve `Some`.
/// - as soon as an argument that doesn't start with either `"--"` or `"-"` is
///   found, the parser enters "positional option mode". it will attempt to
///   greedily give every one of `command`'s arguments that are
///   [`crate::ArgumentForm::Positional`] at most one argument.
/// - when there are no more positional arguments, the parser will check for
///   another argument, and check if it matches any [`crate::Command`] in
///   `command`'s subcommands. if so, this entire process repeats, using the
///   rest of the available arguments in the iterator, and the matched
///   subcommand.
///
/// `argument` may require some manipulation to satisfy type checking.
/// given `&[&str]`, for example:
///
/// ```
/// # const output: purcarg::Output = purcarg::Output::new();
/// # const config: purcarg::Config = purcarg::Config::new();
/// # const command: purcarg::Command<(), ()> = purcarg::Command::new();
/// let arguments_slice: &[&str] = &["--help"];
/// let arguments = arguments_slice.iter().copied();
/// purcarg::parse_str(output, config, command, arguments, ());
/// ```
///
/// `Iterator<Item = String>` may require either collecting or leaking.
///
/// ```
/// # const output: purcarg::Output = purcarg::Output::new();
/// # const config: purcarg::Config = purcarg::Config::new();
/// # const command: purcarg::Command<(), ()> = purcarg::Command::new();
/// let raw_arguments = ["--help".to_string()].into_iter();
/// let arguments = raw_arguments.map(|x| &*x.leak());
/// purcarg::parse_str(output, config, command, arguments, ());
///
/// // or
///
/// let raw_arguments = ["--help".to_string()].into_iter();
/// let arguments_list = raw_arguments.collect::<Vec<_>>();
/// let arguments = arguments_list.iter().map(|x| x.as_str());
/// purcarg::parse_str(output, config, command, arguments, ());
/// ```
///
/// since this currently only accepts `&str`, inputs like
/// `Iterator<Item = &OsStr>` require filtering or otherwise dealing with
/// invalid `str` values.
///
/// ```
/// # const output: purcarg::Output = purcarg::Output::new();
/// # const config: purcarg::Config = purcarg::Config::new();
/// # const command: purcarg::Command<(), ()> = purcarg::Command::new();
/// # use std::ffi::OsStr;
/// let arguments_slice: &[&OsStr] = &[OsStr::new("--help")];
/// let arguments = arguments_slice.iter().filter_map(|x| x.to_str());
/// purcarg::parse_str(output, config, command, arguments, ());
/// ```
///
/// alternatively, use [`crate::parse_bytes()`].
#[expect(clippy::missing_errors_doc, reason = "the error case is self explanatory")]
pub fn parse_str<'a, 'b, T, E>(
	output: Output<'a>,
	config: Config,
	command: Command<T, E>,
	argument: impl IntoIterator<Item = &'b str>,
	layer: T,
) -> Result<Success<T>, Error<'b, E>> {
	let iter = argument.into_iter().map(str::as_bytes);
	parse_bytes(output, config, command, iter, layer)
}

/// parse command line options.
///
/// this is exactly like [`crate::parse_str()`], except this accepts an
/// iterator of `&[u8]` instead of `&str`.
///
/// ```
/// # const output: purcarg::Output = purcarg::Output::new();
/// # const config: purcarg::Config = purcarg::Config::new();
/// # const command: purcarg::Command<(), ()> = purcarg::Command::new();
/// # use std::ffi::OsStr;
/// let arguments_slice: &[&OsStr] = &[OsStr::new("--help")];
/// let arguments = arguments_slice.iter().map(|x| x.as_encoded_bytes());
/// purcarg::parse_bytes(output, config, command, arguments, ());
/// ```
#[expect(clippy::missing_errors_doc, reason = "the error case is self explanatory")]
#[expect(clippy::needless_pass_by_value, reason = "idc")]
pub fn parse_bytes<'a, 'b, T, E>(
	mut output: Output<'a>,
	config: Config,
	command: Command<T, E>,
	argument: impl IntoIterator<Item = &'b [u8]>,
	layer: T,
) -> Result<Success<T>, Error<'b, E>> {
	let mut iter = argument.into_iter().peekable();
	parse_core(&mut output, &config, &command, &mut iter, layer)
}

#[inline]
#[expect(clippy::too_many_lines, reason = "does not need to be shorter")]
fn parse_core<'a, T, E>(
	output: &mut Output<'_>,
	config: &Config,
	command: &Command<T, E>,
	args: &mut core::iter::Peekable<impl Iterator<Item = &'a [u8]>>,
	mut layer: T,
) -> Result<Success<T>, Error<'a, E>> {
	// meow meow meow

	type Mask = u128;

	match command.action {
		Action::None => (),
		Action::LayerBytes(cb) => {
			layer = cb(layer, &mut || args.next())
				.map_err(|e| Error::Other(e))?;
		}
		Action::Layer(cb) => {
			let mut mapped = args.filter_map(|x| str::from_utf8(x).ok());
			layer = cb(layer, &mut || mapped.next())
				.map_err(|e| Error::Other(e))?;
		}
		Action::Help => {
			parse_core_help(command, output, config).map_err(|x| Error::FmtError(x))?;
			return Ok(Success::Help);
		}
		Action::Version => {
			parse_core_version(command, output, config).map_err(|x| Error::FmtError(x))?;
			return Ok(Success::Version);
		}
	}

	// panic here, since this error is entirely the fault of the developer
	assert!(command.argument.len() < Mask::BITS as usize, "too many arguments - max is {}", Mask::BITS);

	let mut required_mask_named = 0 as Mask;

	for (i, arg) in command.argument.iter().enumerate() {
		match arg.form {
			ArgumentForm::Positional(..) => (),
			ArgumentForm::Named { .. } =>
				// unwrap okay, since we asserted arguments is always less than 128
				required_mask_named |= Mask::from(arg.required) << u32::try_from(i).unwrap(),
		}
	}

	// parse options

	let mut required_flag = 0 as Mask;

	while let Some(check) = args.peek() {
		if let Some(long) = check.strip_prefix(b"--") &&
			let Some(long) = str::from_utf8(long).ok()
		{
			args.next();

			// weird situation.
			// we don't know if this argument should be split until after the name is found.
			let (long_check, split) = if let Some(splitter) = config.name_splitting &&
					let Some((new_long, value)) = long.split_once(splitter)
				{
					(new_long, Some(value))
				} else {
					(long, None)
				};

			let (i, arg) = command.argument
				.iter()
				.enumerate()
				.find(|x| match x.1.form {
					ArgumentForm::Named { long: long_list, .. } => long_list.contains(&long_check),
					ArgumentForm::Positional(..) => false,
				})
				.ok_or(Error::BadLong(long_check))?;

			// if splitting *was* disabled for the argument, but splitting was attempted, simply error like nothing happened.
			if matches!(split, Some(..)) && !arg.enable_splitting {
				return Err(Error::BadLong(long));
			}

			let mut passed_args = split.iter().map(|x| x.as_bytes()).chain(args.by_ref());

			match arg.action {
				Action::None => (),
				Action::LayerBytes(cb) => {
					layer = cb(layer, &mut || passed_args.next())
						.map_err(|e| Error::Other(e))?;
				}
				Action::Layer(cb) => {
					let mut mapped = passed_args.filter_map(|x| str::from_utf8(x).ok());
					layer = cb(layer, &mut || mapped.next())
						.map_err(|e| Error::Other(e))?;
				}
				Action::Help => return parse_core_help(command, output, config)
					.map(|()| Success::Help)
					.map_err(|x| Error::FmtError(x)),
				Action::Version => return parse_core_version(command, output, config)
					.map(|()| Success::Version)
					.map_err(|x| Error::FmtError(x)),
			}

			required_flag |= Mask::from(arg.required) << i;
		}
		else if let Some(short_list) = check.strip_prefix(b"-") {
			args.next();

			let mut short_iter = short_list.iter().peekable();

			while let Some(short) = short_iter.next() {
				let short = char::from(*short);

				let (i, arg) = command.argument
					.iter()
					.enumerate()
					.find(|x| match x.1.form {
						ArgumentForm::Named { short: short_list, .. } => short_list.contains(&short),
						ArgumentForm::Positional(..) => false,
					})
					.ok_or(Error::BadShort(short))?;

				match arg.action {
					Action::None => (),
					Action::LayerBytes(cb) =>
						layer = cb(layer, &mut || {
							if short_iter.peek().is_none() {
								args.next()
							}
							else {
								None
							}
						}).map_err(|e| Error::Other(e))?,
					Action::Layer(cb) => {
						let mut mapped = args.filter_map(|x| str::from_utf8(x).ok());
						layer = cb(layer, &mut || {
							if short_iter.peek().is_none() {
								mapped.next()
							}
							else {
								None
							}
						}).map_err(|e| Error::Other(e))?;
					}
					Action::Help => return parse_core_help(command, output, config)
						.map(|()| Success::Help)
						.map_err(|x| Error::FmtError(x)),
					Action::Version => return parse_core_version(command, output, config)
						.map(|()| Success::Version)
						.map_err(|x| Error::FmtError(x)),
				}

				required_flag |= Mask::from(arg.required) << i;
			}
		}
		else {
			break;
		}
	}

	if required_flag != required_mask_named {
		// let's find what's missing!

		// first xor the flags
		let required_xor = required_flag ^ required_mask_named;

		// any left over bit will be from `required_mask_named`. find any:
		let index = 63 - required_xor.leading_zeros();
		let arg = &command.argument[index as usize];

		let error = match arg.form {
			ArgumentForm::Named { long, short } =>
				match (!long.is_empty(), !short.is_empty()) {
					(true, _) => Error::RequiredLong(long[0]),
					(false, true) => Error::RequiredShort(short[0]),
					(false, false) => panic!("argument is required, but no there's name to use it with"),
				}
			ArgumentForm::Positional(..) => unreachable!(),
		};

		return Err(error);
	}

	// parse positionals

	for arg in command.argument
		.iter()
		.filter(|x| x.form.is_positional())
	{
		let value = args.next();

		if value.is_none() {
			if arg.required {
				return Err(Error::RequiredPositional(match arg.form {
					ArgumentForm::Named { .. } => unreachable!(),
					ArgumentForm::Positional(x) => x,
				}));
			}
			// don't give optionals an empty value lol
			continue;
		}

		let mut resetable_value = value.into_iter();

		match arg.action {
			Action::None => (),
			Action::LayerBytes(cb) =>
				layer = cb(layer, &mut || resetable_value.next()).map_err(|e| Error::Other(e))?,
			Action::Layer(cb) => {
				let mut resetable_value =  resetable_value.filter_map(|x| str::from_utf8(x).ok());
				layer = cb(layer, &mut || resetable_value.next()).map_err(|e| Error::Other(e))?;
			}
			Action::Help => return parse_core_help(command, output, config)
				.map(|()| Success::Help)
				.map_err(|x| Error::FmtError(x)),
			Action::Version => return parse_core_version(command, output, config)
				.map(|()| Success::Version)
				.map_err(|x| Error::FmtError(x)),
		}
	}

	// parse subcommand

	if let Some(check) = args.next() {
		if let Some(check) = str::from_utf8(check).ok() &&
			let Some(next_command) = command.subcommand
				.iter()
				.find(|x| x.name.contains(&check))
		{
			return parse_core(output, config, next_command, args, layer);
		}

		// anything left over is an error

		// grab excerpt of what could potentially be a long input
		let positional = str::from_utf8(check).ok().map_or("<unknown>", |x| {
			let mut char_indices = x
				.char_indices()
				.map(|(i, _)| i)
				.chain(core::iter::once(x.len()));
			let end_byte = char_indices
				.nth(31usize)
				.unwrap_or(x.len());
			&x[0..end_byte]
		});

		return Err(Error::BadPositional(positional));
	}

	Ok(Success::Layer(layer))
}

fn parse_core_version<T, E>(
	command: &Command<T, E>,
	output: &mut Output<'_>,
	config: &Config,
) -> Result<(), core::fmt::Error> {
	use core::fmt::Write;

	let Some(ref mut writer) = output.writer else {
		return Ok(());
	};

	writeln!(writer, "{}", command.name.first().copied().unwrap_or("<undefined>"))?;

	if let Some(version) = &config.version {
		write!(writer, " ")?;

		match version {
			ConfigVersion::String(x) => write!(writer, "{}", x)?,
			ConfigVersion::Single(x) => write!(writer, "{}", x)?,
			ConfigVersion::Double(x, y) => write!(writer, "{}.{}", x, y)?,
			ConfigVersion::Triple(x, y, z) => write!(writer, "{}.{}.{}", x, y, z)?,
			ConfigVersion::Quadruple(x, y, z, w) => write!(writer, "{}.{}.{}.{}", x, y, z, w)?,
		}
	}

	Ok(())
}

#[expect(clippy::too_many_lines, reason = "does not need to be shorter")]
fn parse_core_help<T, E>(
	command: &Command<T, E>,
	output: &mut Output<'_>,
	config: &Config,
) -> Result<(), core::fmt::Error> {
	use core::fmt::Write;

	let Some(ref mut writer) = output.writer else {
		return Ok(());
	};

	// usage text

	write!(writer, "usage:")?;

	let has_subcommand = !command.subcommand.is_empty();
	let has_positional = command.argument.iter().any(|x| x.form.is_positional());
	let has_named = command.argument.iter().any(|x| x.form.is_named());

	if has_named {
		write!(writer, " <options>")?;
	}

	if has_positional {
		if has_subcommand {
			write!(writer, " (<subcommand> | ")?;
		}

		let positional_iter = command.argument
			.iter()
			.filter_map(|x| match x.form {
				ArgumentForm::Positional(y) => Some((x, y)),
				ArgumentForm::Named { .. } => None,
			});

		write_joined(writer, positional_iter, " ", |w, (arg, name)| {
			if arg.required {
				write!(w, "<{}>", name)
			}
			else {
				write!(w, "[<{}>]", name)
			}
		})?;

		if has_subcommand {
			write!(writer, ")")?;
		}
	}
	else if has_subcommand {
		write!(writer, " <subcommand>")?;
	}

	writeln!(writer)?;

	// description

	if let Some(print) = match (command.description, command.help) {
		| (Some(a), _)
		| (None, Some(a)) => Some(a),
		(None, None) => None,
	} {
		writeln!(writer, "{}", print)?;
		writeln!(writer)?;
	}

	// options

	if has_positional || has_named {
		writeln!(writer, "options:")?;

		for (arg, long, short) in command.argument
			.iter()
			.filter_map(|x| {
				match x.form {
					ArgumentForm::Positional(..) => None,
					ArgumentForm::Named { long, short } => Some((x, long, short)),
				}
			})
		{
			write!(writer, "  ")?;

			if !short.is_empty() || !long.is_empty() {
				if !short.is_empty() {
					write_joined(writer, short.iter(), " or ", |w, x| {
						write!(w, "-{}", x)
					})?;

					if !long.is_empty() {
						write!(writer, ", ")?;
					}
				}

				if !long.is_empty() {
					if short.is_empty() {
						write!(writer, "    ")?;
					}

					write_joined(writer, long.iter(), " or ", |w, x| {
						write!(w, "--{}", x)
					})?;
				}

				writeln!(writer)?;
			}
			else {
				writeln!(writer, "<undefined>")?;
			}

			if let Some(help) = arg.help {
				write_wrapped(writer, help, "         ", config.wrap)?;
			}
		}
	}

	if has_subcommand {
		writeln!(writer, "subcommands:")?;

		for subcommand in command.subcommand {
			write!(writer, "  ")?;

			if subcommand.name.is_empty() {
				writeln!(writer, "<undefined>")?;
			}
			else {
				write_joined(writer, subcommand.name.iter(), " or ", |w, x| {
					write!(w, "{}", x)
				})?;

				writeln!(writer)?;
			}

			if let Some(help) = subcommand.help {
				write_wrapped(writer, help, "   ", config.wrap)?;
			}
		}

		writeln!(writer)?;
	}

	Ok(())
}

fn write_joined<T>(
	writer: &mut dyn core::fmt::Write,
	iter: impl IntoIterator<Item = T>,
	sep: &str,
	mut f: impl FnMut(&mut dyn core::fmt::Write, T) -> core::fmt::Result,
) -> core::fmt::Result {
	let mut iter = iter.into_iter().peekable();

	while let Some(item) = iter.next() {
		f(writer, item)?;

		if iter.peek().is_some() {
			write!(writer, "{}", sep)?;
		}
	}

	Ok(())
}

fn write_wrapped(
	writer: &mut dyn core::fmt::Write,
	text: &str,
	indent: &str,
	width: Option<u32>,
) -> core::fmt::Result {
	let mut col = 0;

	write!(writer, "{}", indent)?;

	let Some(width) = width else {
		return writeln!(writer, "{}", text);
	};

	let mut iter = text.split_whitespace().peekable();

	while let Some(word) = iter.next() {
		let len = word.len() + 1;

		if col + len > width as usize {
			writeln!(writer)?;
			write!(writer, "{}", indent)?;
			col = 0;
		}

		write!(writer, "{}", word)?;

		if let Some(check) = iter.peek() &&
			col + len + check.len() <= width as usize {
			write!(writer, " ")?;
		}

		col += len;
	}

	writeln!(writer)
}
