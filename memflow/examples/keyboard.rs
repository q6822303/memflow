use clap::{crate_authors, crate_version, App, Arg};
use log::Level;
/// A simple keyboard example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let (conn_name, conn_args, os_name, os_args) = parse_args()?;

    // create connector + os
    let inventory = Inventory::scan();
    let os = inventory
        .builder()
        .connector(&conn_name)
        .args(conn_args)
        .os(&os_name)
        .args(os_args)
        .build()?;

    if !os.check_impl_oskeyboardinner() {
        return Err(
            Error(ErrorOrigin::Other, ErrorKind::UnsupportedOptionalFeature)
                .log_error("keyboard feature is not implemented for the given os plugin"),
        );
    }

    let mut keyboard = into!(os impl OsKeyboardInner).unwrap().into_keyboard()?;

    loop {
        println!("space down: {:?}", keyboard.is_down(0x20)); // VK_SPACE
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn parse_args() -> Result<(String, ConnectorArgs, String, OsArgs)> {
    let matches = App::new("mfps")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("connector-args")
                .long("connector-args")
                .short("x")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("os-args")
                .long("os-args")
                .short("y")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    // set log level
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };

    // initialize loggong
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();

    Ok((
        matches
            .value_of("connector")
            .ok_or_else(|| {
                Error(ErrorOrigin::Other, ErrorKind::Configuration)
                    .log_error("failed to parse connector")
            })?
            .into(),
        str::parse(matches.value_of("connector-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration)
                .log_error("failed to parse connector args")
        })?)?,
        matches
            .value_of("os")
            .ok_or_else(|| {
                Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os")
            })?
            .into(),
        str::parse(matches.value_of("os-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os args")
        })?)?,
    ))
}
