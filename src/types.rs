/// defines the cli's shape.
///
/// this api is based around [`crate::Action`], namely
/// [`crate::Action::Layer`]. as described in [`crate::parse()`], for each
/// option the cli user specifies, `T` is passed into each option's respective
/// 'layer', manipulated, then returned to the next matched layer.
///
/// the main vector for this is found in [`crate::Argument`], via
/// [`field@Self::action`]. consider the following example:
///
/// ```
/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
///     .argument(&[
///         purcarg::Argument::new()
///             .named(&["list"], &['l'])
///             .action_layer(|input, next| {
///                 Ok(input)
///             })
///     ]);
/// ```
///
/// this specifies a cli with one named option, `--list` (with a short form
/// `-l`). if a user passes in `"--list"` into the parser, that option's
/// `Action` will match, running the function.
///
///
#[derive(Debug, PartialEq)]
pub struct Command<T: 'static, E: 'static> {
	/// subcommand names. not particularly useful for the root `Command`.
	pub name: &'static [&'static str],
	/// description of the command to be used in the help page.
	pub description: Option<&'static str>,
	/// like [`field@Self::description`], except more so intended as a brief
	/// summary used in the help page's subcommand list.
	pub help: Option<&'static str>,
	/// action to run if the `Command` matches. for the root `Command`,
	/// this always matches.
	pub action: Action<T, E>,
	/// list of options. see: [`crate::Argument`].
	pub argument: &'static [Argument<T, E>],
	/// list of subcommands.
	pub subcommand: &'static [Command<T, E>],
}

impl<T, E> Clone for Command<T, E> {
	fn clone(&self) -> Self {
		// kill me just fucking kill me
		Self {
			name: self.name,
			description: self.description,
			help: self.help,
			action: self.action.clone(),
			argument: self.argument,
			subcommand: self.subcommand,
		}
	}
}

impl<T, E> Default for Command<T, E> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, E> Command<T, E> {
	/// constructs a new [`Self`].
	///
	/// ```
	/// const command: purcarg::Command::<(), ()> = purcarg::Command::new();
	///
	/// assert_eq!(command, purcarg::Command {
	///     name: &[],
	///     description: None,
	///     help: None,
	///     action: purcarg::Action::None,
	///     argument: &[],
	///     subcommand: &[],
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self {
			name: &[],
			description: None,
			help: None,
			action: Action::None,
			argument: &[],
			subcommand: &[],
		}
	}

	/// sets [`field@Self::name`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .name(&["example", "e"]);
	///
	/// assert_eq!(command, purcarg::Command {
	///     name: &["example", "e"],
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn name(mut self, names: &'static [&'static str]) -> Self {
		self.name = names;
		self
	}

	/// sets [`field@Self::description`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .description("example");
	///
	/// assert_eq!(command, purcarg::Command {
	///     description: Some("example"),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn description(mut self, description: &'static str) -> Self {
		self.description = Some(description);
		self
	}

	/// sets [`field@Self::help`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .help("example");
	///
	/// assert_eq!(command, purcarg::Command {
	///     help: Some("example"),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn help(mut self, help: &'static str) -> Self {
		self.help = Some(help);
		self
	}

	#[must_use]
	#[inline]
	const fn action_raw(mut self, action: Action<T, E>) -> Self {
		self.action = action;
		self
	}

	/// sets [`field@Self::action`] to [`crate::Action::None`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .action_none();
	///
	/// assert_eq!(command, purcarg::Command {
	///     action: purcarg::Action::None,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_none(self) -> Self {
		self.action_raw(Action::None)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Help`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .action_help();
	///
	/// assert_eq!(command, purcarg::Command {
	///     action: purcarg::Action::Help,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_help(self) -> Self {
		self.action_raw(Action::Help)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Version`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .action_version();
	///
	/// assert_eq!(command, purcarg::Command {
	///     action: purcarg::Action::Version,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_version(self) -> Self {
		self.action_raw(Action::Version)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Layer`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .action_layer(|_, _| Ok(()));
	///
	/// assert_eq!(command, purcarg::Command {
	///     action: purcarg::Action::Layer(|_, _| Ok(())),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_layer(self, layer: Layer<T, E>) -> Self {
		self.action_raw(Action::Layer(layer))
	}

	/// sets [`field@Self::action`] to [`crate::Action::LayerBytes`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .action_layer_bytes(|_, _| Ok(()));
	///
	/// assert_eq!(command, purcarg::Command {
	///     action: purcarg::Action::LayerBytes(|_, _| Ok(())),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_layer_bytes(self, layer: LayerBytes<T, E>) -> Self {
		self.action_raw(Action::LayerBytes(layer))
	}

	/// sets [`field@Self::argument`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .argument(&[purcarg::Argument::new()]);
	///
	/// assert_eq!(command, purcarg::Command {
	///     argument: &[const { purcarg::Argument::new() }],
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn argument(mut self, arguments: &'static [Argument<T, E>]) -> Self {
		self.argument = arguments;
		self
	}

	/// sets [`field@Self::subcommand`].
	///
	/// ```
	/// const command: purcarg::Command<(), ()> = purcarg::Command::new()
	///     .subcommand(&[purcarg::Command::new()]);
	///
	/// assert_eq!(command, purcarg::Command {
	///     subcommand: &[const { purcarg::Command::new() }],
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn subcommand(mut self, subcommands: &'static [Command<T, E>]) -> Self {
		self.subcommand = subcommands;
		self
	}
}

/// [`crate::Command`] arguments.
#[derive(Debug, PartialEq)]
pub struct Argument<T, E> {
	/// what kind of option is this?
	/// see: [`crate::ArgumentForm`].
	pub form: ArgumentForm,
	/// description of the option to be used in the help page.
	pub help: Option<&'static str>,
	/// what to do when this option matches?
	/// see: [`crate::Action`].
	pub action: Action<T, E>,
	/// whether this option is required or not.
	pub required: bool,
	/// whether to enable name splitting (`--name=value`) for this argument.
	/// only used if [`field@Self::form`] is [`crate::ArgumentForm::Named`].
	pub enable_splitting: bool,
}

impl<T, E> Clone for Argument<T, E> {
	fn clone(&self) -> Self {
		Self {
			form: self.form.clone(),
			help: self.help,
			action: self.action.clone(),
			required: self.required,
			enable_splitting: self.enable_splitting,
		}
	}
}

impl<T, E> Default for Argument<T, E> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T, E> Argument<T, E> {
	/// constructs a new [`Self`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new();
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     form: purcarg::ArgumentForm::Positional("unknown"),
	///     help: None,
	///     action: purcarg::Action::None,
	///     required: false,
	///     enable_splitting: false,
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self {
			form: ArgumentForm::Positional("unknown"),
			help: None,
			action: Action::None,
			required: false,
			enable_splitting: false,
		}
	}

	/// sets [`field@Self::form`] to [`crate::ArgumentForm::Positional`].
	///
	/// `name` is only used in the help page.
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .positional("example");
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     form: purcarg::ArgumentForm::Positional("example"),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn positional(mut self, name: &'static str) -> Self {
		self.form = ArgumentForm::Positional(name);
		self
	}

	/// sets [`field@Self::form`] to [`crate::ArgumentForm::Named`].
	///
	/// `long` correspond to `--name` style options, and `short` correspond to
	/// `-n` style options setting both to empty arrays is possible, but will
	/// leave the argument unmatchable.
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .named(&["example"], &['e']);
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     form: purcarg::ArgumentForm::Named { long: &["example"], short: &['e'] },
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn named(mut self, long: &'static [&'static str], short: &'static [char]) -> Self {
		self.form = ArgumentForm::Named { long, short };
		self
	}

	/// sets [`field@Self::help`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .help("example");
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     help: Some("example"),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn help(mut self, help: &'static str) -> Self {
		self.help = Some(help);
		self
	}

	#[must_use]
	#[inline]
	const fn action_raw(mut self, action: Action<T, E>) -> Self {
		self.action = action;
		self
	}

	/// sets [`field@Self::action`] to [`crate::Action::None`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .action_none();
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     action: purcarg::Action::None,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_none(self) -> Self {
		self.action_raw(Action::None)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Help`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .action_help();
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     action: purcarg::Action::Help,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_help(self) -> Self {
		self.action_raw(Action::Help)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Version`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .action_version();
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     action: purcarg::Action::Version,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_version(self) -> Self {
		self.action_raw(Action::Version)
	}

	/// sets [`field@Self::action`] to [`crate::Action::Layer`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .action_layer(|_, _| Ok(()));
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     action: purcarg::Action::Layer(|_, _| Ok(())),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_layer(self, layer: Layer<T, E>) -> Self {
		self.action_raw(Action::Layer(layer))
	}

	/// sets [`field@Self::action`] to [`crate::Action::LayerBytes`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .action_layer_bytes(|_, _| Ok(()));
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     action: purcarg::Action::LayerBytes(|_, _| Ok(())),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn action_layer_bytes(self, layer: LayerBytes<T, E>) -> Self {
		self.action_raw(Action::LayerBytes(layer))
	}

	/// sets [`field@Self::required`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .required(true);
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     required: true,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn required(mut self, require: bool) -> Self {
		self.required = require;
		self
	}

	/// sets [`field@Self::enable_splitting`].
	///
	/// ```
	/// const argument: purcarg::Argument<(), ()> = purcarg::Argument::new()
	///     .splitting(true);
	///
	/// assert_eq!(argument, purcarg::Argument {
	///     enable_splitting: true,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn splitting(mut self, split: bool) -> Self {
		self.enable_splitting = split;
		self
	}
}

/// specifies how the argument is used. used in [`field@crate::Argument::form`].
#[derive(Debug, Clone, PartialEq)]
pub enum ArgumentForm {
	/// positional argument. always matches if input is available.
	/// the string here is the name of the positional, used for the help page.
	Positional(&'static str),
	/// named argument. only matches if either `--[long]` or `-[short]` is found.
	Named {
		/// list of long names.
		long: &'static [&'static str],
		/// list of short names.
		short: &'static [char],
	},
}

impl ArgumentForm {
	#[inline]
	pub(crate) fn is_positional(&self) -> bool {
		!self.is_named()
	}

	#[inline]
	pub(crate) fn is_named(&self) -> bool {
		matches!(self, Self::Named { .. })
	}
}

type Layer<T, E> = for<'a> fn(config: T, next: &mut dyn FnMut() -> Option<&'a str>) -> Result<T, E>;
type LayerBytes<T, E> = for<'a> fn(config: T, next: &mut dyn FnMut() -> Option<&'a [u8]>) -> Result<T, E>;

/// what to do if a command or option matches?
/// see [`crate::Command`] for more information.
#[derive(Debug)]
pub enum Action<T, E> {
	/// when this action matches, nothing happens.
	None,
	/// when this action matches, the supplied
	/// function allows you to transform the input.
	///
	/// `next()` returns `None` if either there isn't another
	/// argument available, or if the next argument is not valid utf-8.
	/// calling this function always consumes the next
	/// argument, even if utf-8 validation fails.
	Layer(Layer<T, E>),
	/// when this action matches, the supplied
	/// function allows you to transform the input.
	///
	/// `next()` returns `None` if either there isn't another argument available.
	LayerBytes(LayerBytes<T, E>),
	/// when this action matches, a generated help page is written to
	/// whatever is specified by [`crate::Output`], and parsing stops.
	Help,
	/// when this action matches, a generated version page is written to
	/// whatever is specified by [`crate::Output`], and parsing stops.
	Version,
}

impl<T, E> PartialEq for Action<T, E> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		matches!(
			(self, other),
			| (Self::None, Self::None)
			| (Self::Layer(..), Self::Layer(..))
			| (Self::LayerBytes(..), Self::LayerBytes(..))
			| (Self::Help, Self::Help)
			| (Self::Version, Self::Version)
		)
	}
}

impl<T, E> Clone for Action<T, E> {
	#[inline]
	fn clone(&self) -> Self {
		match self {
			Self::Help => Self::Help,
			Self::None => Self::None,
			Self::Layer(x) => Self::Layer(*x),
			Self::LayerBytes(x) => Self::LayerBytes(*x),
			Self::Version => Self::Version,
		}
	}
}

/// configuration unrelated to the direct operation of
/// the parser - namely, where to write the parser's output to.
///
/// it should be noted that the only time parser will write to this
/// is when [`Action::Help`] or similar is matched. when an error is
/// found, the parser exits immediately - you still have to write
/// the error yourself.
#[derive(Debug, PartialEq)]
pub struct Output<'a> {
	/// where to write the parser's output to.
	pub writer: Option<OutputWriter<'a>>,
}

impl Default for Output<'_> {
	fn default() -> Self {
		Self::new()
	}
}

impl<'a> Output<'a> {
	/// constructs a new [`crate::Output`].
	///
	/// ```
	/// let output = purcarg::Output::new();
	///
	/// assert_eq!(output, purcarg::Output {
	///     writer: None,
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self {
			writer: None,
		}
	}

	#[must_use]
	#[inline]
	const fn writer_raw(mut self, writer: OutputWriter<'a>) -> Self {
		self.writer = Some(writer);
		self
	}

	/// set to write to a `dyn core::fmt::Write`.
	/// see: [`field@Self::writer`].
	///
	/// ```
	/// let mut string = String::new();
	/// let output = purcarg::Output::new()
	///     .writer_fmt(&mut string);
	///
	/// assert!(matches!(output.writer, Some(purcarg::OutputWriter::Fmt(..))));
	/// ```
	#[must_use]
	#[inline]
	pub const fn writer_fmt(self, write: &'a mut dyn core::fmt::Write) -> Self {
		self.writer_raw(OutputWriter::Fmt(write))
	}

	/// set to write to a `dyn std::io::Write`.
	/// see: [`field@Self::writer`].
	///
	/// ```
	/// let mut stdio = std::io::stdout();
	/// let output = purcarg::Output::new()
	///     .writer_io(&mut stdio);
	///
	/// assert!(matches!(output.writer, Some(purcarg::OutputWriter::Io(..))));
	/// ```
	#[cfg(feature = "std")]
	#[must_use]
	#[inline]
	pub const fn writer_io(self, write: &'a mut dyn std::io::Write) -> Self {
		self.writer_raw(OutputWriter::Io(write))
	}

	/// set to write directly to stdout.
	/// see: [`field@Self::writer`].
	///
	/// ```
	/// let output = purcarg::Output::new()
	///     .writer_stdio();
	///
	/// assert!(matches!(output.writer, Some(purcarg::OutputWriter::Stdio)));
	/// ```
	#[cfg(feature = "std")]
	#[must_use]
	#[inline]
	pub const fn writer_stdio(self) -> Self {
		self.writer_raw(OutputWriter::Stdio)
	}

	/// set to write to a `String`.
	/// see: [`field@Self::writer`].
	///
	/// ```
	/// let mut string = String::new();
	/// let output = purcarg::Output::new()
	///     .writer_string(&mut string);
	///
	/// assert!(matches!(output.writer, Some(purcarg::OutputWriter::String(..))));
	/// ```
	#[cfg(feature = "std")]
	#[must_use]
	#[inline]
	pub const fn writer_string(self, string: &'a mut std::string::String) -> Self {
		self.writer_raw(OutputWriter::String(string))
	}
}

/// configures where output (from the generated
/// help or version page) should be written to.
///
/// in `no_std`, only the [`Self::Fmt`] variant is defined.
pub enum OutputWriter<'a> {
	/// writes to a `dyn core::fmt::Write`.
	///
	/// ```
	/// let mut string = String::new();
	/// let writer = purcarg::OutputWriter::Fmt(&mut string);
	/// ```
	Fmt(&'a mut dyn core::fmt::Write),
	/// writes to a `dyn std::io::Write`.
	/// available when the `std` feature is enabled.
	///
	/// ```
	/// let mut stdio = std::io::stdout();
	/// let writer = purcarg::OutputWriter::Io(&mut stdio);
	/// ```
	#[cfg(feature = "std")]
	Io(&'a mut dyn std::io::Write),
	/// writes to stdout. currently implemented with a normal `println!()`.
	/// available when the `std` feature is enabled.
	///
	/// ```
	/// let writer = purcarg::OutputWriter::Stdio;
	/// ```
	#[cfg(feature = "std")]
	Stdio,
	/// writes to a string.
	/// available when the `std` feature is enabled.
	///
	/// ```
	/// let mut string = String::new();
	/// let writer = purcarg::OutputWriter::String(&mut string);
	/// ```
	#[cfg(feature = "std")]
	String(&'a mut std::string::String),
}

impl core::fmt::Debug for OutputWriter<'_> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Fmt(..) => write!(f, "OutputWriter::Fmt"),
			#[cfg(feature = "std")]
			Self::Io(..) => write!(f, "OutputWriter::Io"),
			#[cfg(feature = "std")]
			Self::Stdio => write!(f, "OutputWriter::Stdio"),
			#[cfg(feature = "std")]
			Self::String(..) => write!(f, "OutputWriter::String"),
		}
	}
}

impl PartialEq for OutputWriter<'_> {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Fmt(..), Self::Fmt(..)) => true,
			#[cfg(feature = "std")]
			| (Self::Io(..), Self::Io(..))
			| (Self::Stdio, Self::Stdio) => true,
			| (Self::String(..), Self::String(..)) => true,
			_ => false,
		}
	}
}

impl core::fmt::Write for OutputWriter<'_> {
	fn write_str(&mut self, s: &str) -> core::fmt::Result {
		match self {
			OutputWriter::Fmt(x) =>
				x.write_str(s),
			#[cfg(feature = "std")]
			OutputWriter::Io(x) =>
				x.write_all(s.as_bytes())
					.map_err(|_| core::fmt::Error),
			#[cfg(feature = "std")]
			OutputWriter::Stdio => {
				std::println!("{}", s);
				Ok(())
			}
			#[cfg(feature = "std")]
			OutputWriter::String(x) => {
				x.push_str(s);
				Ok(())
			}
		}
	}
}

/// configure the parser's metadata, parsing rules, help and version
/// formatting, etc.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
	/// version of the cli the generated version page will report.
	/// see: [`crate::ConfigVersion`].
	pub version: Option<ConfigVersion>,
	/// specifies how many characters wide a block of text can be
	/// until it wraps. `None` to disable wrapping.
	pub wrap: Option<u32>,
	/// what character to use when name splitting (eg. `--name=value` or
	/// `--name:value`).
	/// `None` to disable.
	pub name_splitting: Option<&'static str>,
}

impl Default for Config {
	fn default() -> Self {
		Self::new()
	}
}

impl Config {
	/// constructs a new [`Self`].
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new();
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: None,
	///     wrap: Some(60),
	///     name_splitting: None,
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn new() -> Self {
		Self {
			version: None,
			wrap: Some(60),
			name_splitting: None,
		}
	}

	#[must_use]
	#[inline]
	const fn version_raw(mut self, version: ConfigVersion) -> Self {
		self.version = Some(version);
		self
	}

	/// sets [`field@Self::version`] to [`crate::ConfigVersion::Single`].
	///
	/// equivalent to
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .version_single(1); // 1
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: Some(purcarg::ConfigVersion::Single(1)),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn version_single(self, x: u32) -> Self {
		self.version_raw(ConfigVersion::Single(x))
	}

	/// sets [`field@Self::version`].
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .version_double(1, 2); // 1.2
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: Some(purcarg::ConfigVersion::Double(1, 2)),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn version_double(self, x: u32, y: u32) -> Self {
		self.version_raw(ConfigVersion::Double(x, y))
	}

	/// sets [`field@Self::version`].
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .version_triple(1, 2, 3); // "1.2.3"
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: Some(purcarg::ConfigVersion::Triple(1, 2, 3)),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn version_triple(self, x: u32, y: u32, z: u32) -> Self {
		self.version_raw(ConfigVersion::Triple(x, y, z))
	}

	/// sets [`field@Self::version`].
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .version_quadruple(1, 2, 3, 4); // "1.2.3.4"
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: Some(purcarg::ConfigVersion::Quadruple(1, 2, 3, 4)),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn version_quadruple(self, x: u32, y: u32, z: u32, w: u32) -> Self {
		self.version_raw(ConfigVersion::Quadruple(x, y, z, w))
	}


	/// sets [`field@Self::version`].
	///
	/// equivalent to
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .version_string("1.2.3-beta"); // "1.2.3-beta"
	///
	/// assert_eq!(config, purcarg::Config {
	///     version: Some(purcarg::ConfigVersion::String("1.2.3-beta")),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn version_string(self, string: &'static str) -> Self {
		self.version_raw(ConfigVersion::String(string))
	}

	/// sets [`field@Self::wrap`]. disables text wrapping.
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .nowrap();
	///
	/// assert_eq!(config, purcarg::Config {
	///     wrap: None,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn nowrap(mut self) -> Self {
		self.wrap = None;
		self
	}

	/// sets [`field@Self::wrap`]. enables text wrapping.
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .wrap(40);
	///
	/// assert_eq!(config, purcarg::Config {
	///     wrap: Some(40),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn wrap(mut self, wrap: u32) -> Self {
		self.wrap = Some(wrap);
		self
	}

	/// sets [`field@Self::name_splitting`]. disables name splitting.
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .nosplitting();
	///
	/// assert_eq!(config, purcarg::Config {
	///     name_splitting: None,
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn nosplitting(mut self) -> Self {
		self.name_splitting = None;
		self
	}

	/// sets [`field@Self::name_splitting`]. enables name splitting.
	///
	/// ```
	/// const config: purcarg::Config = purcarg::Config::new()
	///     .splitting("=");
	///
	/// assert_eq!(config, purcarg::Config {
	///     name_splitting: Some("="),
	///     ..Default::default()
	/// });
	/// ```
	#[must_use]
	#[inline]
	pub const fn splitting(mut self, splitter: &'static str) -> Self {
		self.name_splitting = Some(splitter);
		self
	}
}

/// used with [`crate::Config`] to configure how the cli's version should be
/// rendered.
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigVersion {
	/// formatted as `"x"`.
	Single(u32),
	/// formatted as `"x.y"`.
	Double(u32, u32),
	/// formatted as `"x.y.z"`.
	Triple(u32, u32, u32),
	/// formatted as `"x.y.z.w"`.
	Quadruple(u32, u32, u32, u32),
	/// custom format.
	String(&'static str),
}

/// an error type returned by the [`crate::parse()`] functions.
#[derive(Debug, Clone, PartialEq)]
pub enum Error<'a, E> {
	/// emitted when an unknown or malformed long option (`--name`) is found.
	BadLong(&'a str),
	/// emitted when an unknown or malformed short option (`-n`) is found.
	BadShort(char),
	/// emitted when an unexpected positional value is found.
	/// the included `&str` may be truncated.
	BadPositional(&'a str),
	/// emitted when, after parsing all the arguments command, any option marked
	/// as required wasn't specified.
	///
	/// when such an option is found, this variant is emitted if a long
	/// option (`--name`) exists for the command. if only a short option (`-n`)
	/// exists for the command, [`crate::Error::RequiredShort`] is
	/// emitted instead.
	RequiredLong(&'static str),
	/// emitted when, after parsing all the arguments command, any option marked
	/// as required wasn't specified.
	///
	/// specifically, this is only emitted if the related command didn't
	/// have a long argument (`--name`) defined, but only a short one (`-n`).
	RequiredShort(char),
	/// emitted when, after parsing all the arguments, any positional option
	/// marked was required wasn't found.
	RequiredPositional(&'static str),
	/// emitted if an error occurs while writing the help or version page.
	/// internally, this library only interacts with [`core::fmt::Write`],
	/// even if [`crate::Output`] was set to a [`std::io::Write`] type, and
	/// therefore can only return [`core::fmt::Error`].
	FmtError(core::fmt::Error),
	/// emitted if a [`crate::Action`] transformation function returns `Err`.
	Other(E),
}

impl<E: core::fmt::Debug + core::fmt::Display> core::fmt::Display for Error<'_, E> {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::BadLong(x) => write!(f, "unknown argument specifier '{}'", x),
			Self::BadShort(x) => write!(f, "unknown short specifier '{}'", x),
			Self::BadPositional(x) => write!(f, "unexpected value '{}'", x),
			Self::RequiredLong(x) => write!(f, "argument '--{}' unspecified, but required", x),
			Self::RequiredShort(x) => write!(f, "argument '-{}' unspecified, but required", x),
			Self::RequiredPositional(x) => write!(f, "positional argument {} missing, but required", x),
			Self::FmtError(x) => write!(f, "{}", x),
			Self::Other(x) => write!(f, "{}", x),
		}
	}
}

impl<E: core::fmt::Debug + core::fmt::Display> core::error::Error for Error<'_, E> {}
