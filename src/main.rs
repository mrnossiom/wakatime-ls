//! Wakatime LS

use std::{
	env,
	io::ErrorKind,
	panic::{self, PanicHookInfo},
	process::{self, Command},
};
use tokio::runtime::Builder;
use tower_lsp_server::{LspService, Server};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan};
use wakatime_ls::Backend;

/// Transfers panic messages to the tracing logging pipeline
fn tracing_panic_hook(panic_info: &PanicHookInfo) {
	let payload = panic_info
		.payload()
		.downcast_ref::<&'static str>()
		.map_or_else(
			|| {
				panic_info
					.payload()
					.downcast_ref::<String>()
					.map_or("Box<dyn Any>", |s| &s[..])
			},
			|s| *s,
		);

	let location = panic_info.location().map(ToString::to_string);

	tracing::error!(
		panic.payload = payload,
		panic.location = location,
		"A panic occurred",
	);
}

/// Entrypoint when running with no arguments
async fn ls_main() {
	let file_appender = tracing_appender::rolling::never("/tmp", "wakatime-ls.log");
	let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
	tracing_subscriber::fmt()
		.with_writer(non_blocking)
		.with_span_events(FmtSpan::NEW)
		.with_env_filter(EnvFilter::from_default_env())
		.init();

	let stdin = tokio::io::stdin();
	let stdout = tokio::io::stdout();

	let (service, socket) = LspService::new(Backend::new);
	Server::new(stdin, stdout, socket).serve(service).await;
}

fn main() {
	panic::set_hook(Box::new(tracing_panic_hook));

	let mut args = env::args();
	let _binary = args.next();

	if let Some(arg) = args.next() {
		match arg.as_str() {
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
				// TODO: add a health check for api key
			}
			option => {
				println!("invalid option `{option}` provided");
				println!("usage: {} [--version | --health]", env!("CARGO_PKG_NAME"));
			}
		}
		process::exit(0);
	}

	// We really don't need much power with what we are doing
	let rt = Builder::new_current_thread()
		.build()
		.expect("config is valid");
	rt.block_on(ls_main());
}
