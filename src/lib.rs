#[macro_use]
extern crate serde_derive;
extern crate console;
extern crate docopt;
extern crate flexi_logger;
extern crate serde;

pub use console::{style, Style, StyledObject};
use docopt::Docopt;
use flexi_logger::{Level, LevelFilter, LogSpecification, Logger, Record};
use serde::de::Deserialize;
use std::io;

#[macro_export]
macro_rules! cargo_version {
    // `()` indicates that the macro takes no argument.
    () => {
        // The macro will expand into the contents of this block.
        format!(
            "{}.{}.{}{}",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH"),
            option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
        );
    };
}

#[derive(PartialEq, Deserialize, Debug)]
pub enum ColorOption {
    Auto,
    Always,
    Never,
}

pub trait Options {
    fn debug(&self) -> bool {
        self.verbose()
    }

    fn verbose(&self) -> bool {
        false
    }

    fn color(&self) -> &ColorOption {
        &ColorOption::Auto
    }
}

pub fn get_options<'a, T>(usage: &str) -> io::Result<T>
where
    T: Deserialize<'a> + Options,
{
    Docopt::new(usage)
        .and_then(|d| Ok(d.version(Some(cargo_version!()))))
        .and_then(|d| Ok(d.options_first(true)))
        .and_then(|d| d.deserialize())
        .map_err(|e| e.exit())
        .and_then(|o| {
            set_colors_enabled(&o);
            set_loglevel(&o);
            Ok(o)
        })
}

fn set_colors_enabled<T>(options: &T)
where
    T: Options,
{
    match *options.color() {
        ColorOption::Never => console::set_colors_enabled(false),
        ColorOption::Always => console::set_colors_enabled(true),
        ColorOption::Auto => (),
    };
}

fn set_loglevel<T>(options: &T)
where
    T: Options,
{
    let log_spec_builder = if options.debug() {
        LogSpecification::default(LevelFilter::max())
    } else if options.verbose() {
        LogSpecification::default(LevelFilter::Info)
    } else {
        LogSpecification::default(LevelFilter::Warn)
    };

    let log_spec = log_spec_builder.build();

    Logger::with(log_spec).format(format_logs).start().unwrap();
}

fn format_logs(writer: &mut io::Write, record: &Record) -> Result<(), io::Error> {
    let style = match record.level() {
        Level::Trace => Style::new().white().dim().italic(),
        Level::Debug => Style::new().white().dim(),
        Level::Info => Style::new().white(),
        Level::Warn => Style::new().yellow(),
        Level::Error => Style::new().red(),
    };

    writer
        .write(&format!("{}", style.apply_to(record.args())).into_bytes())
        .map(|_| ())
}
