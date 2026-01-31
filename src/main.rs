//! Wakatime LS

use lsp_server::Connection;
use std::{
	env,
	io::ErrorKind,
	process::{self, Command},
};
use wakatime_ls::LanguageServer;
mod utils;

const USAGE: &str = concat!(
	"usage: ",
	env!("CARGO_PKG_NAME"),
	" [--help | --version | --health]"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Easier to see in editor logs if the current development version is loaded
	#[cfg(debug_assertions)]
	eprintln!("DEBUG VERSION");

	let mut args = env::args();
	let _binary = args.next();

	if let Some(arg) = args.next() {
		match arg.as_str() {
			"--help" => println!("{USAGE}"),
			"--version" => println!("{}", env!("CARGO_PKG_VERSION")),
			"--health" => {
				// `wakatime-ls` version
				println!("{}: {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

				// `wakatime-cli` version if found or notice
				match Command::new("wakatime-cli").arg("--version").output() {
					Ok(output) => println!(
						"wakatime-cli: {}",
						String::from_utf8_lossy(output.stdout.trim_ascii())
					),
					Err(err) if err.kind() == ErrorKind::NotFound => {
						println!("wakatime-cli: not found in path (wakatime-ls needs it)");
					}
					Err(err) => {
						eprintln!("could not execute `wakatime-cli`: {err:?}");
					}
				}

				match Command::new("wakatime-cli")
					.arg("--config-read")
					.arg("api_key")
					.output()
				{
					Ok(output) => {
						if utils::is_valid_api_key(
							String::from_utf8_lossy(output.stdout.trim_ascii()).into_owned(),
						) {
							println!("wakatime-cli api-key: \u{2705}");
						} else {
							println!(
								"wakatime-cli api-key: \u{274C} (Please add a valid api-key in wakatime config file)"
							);
						}
					}
					Err(err) if err.kind() == ErrorKind::NotFound => {
						println!("wakatime-cli: not found in path (wakatime-ls needs it)");
					}
					Err(err) => {
						eprintln!("could not execute `wakatime-cli`: {err:?}");
					}
				}
			}
			option => {
				println!("invalid option `{option}` provided\n{USAGE}");
			}
		}
		process::exit(0);
	}

	// Create the transport. Includes the stdio (stdin and stdout) versions but
	// this could also be implemented to use sockets or HTTP.
	let (connection, io_threads) = Connection::stdio();

	LanguageServer::new(connection).start()?;

	io_threads.join()?;

	Ok(())
}
