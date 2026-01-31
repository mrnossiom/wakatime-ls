//! Wakatime LS implementation
//!
//! Entrypoint is [`Backend::new`]

// TODO: check options for additional ideas <https://github.com/wakatime/wakatime-cli/blob/develop/USAGE.md#ini-config-file>

// TODO: implement debouncing ourselves to avoid wkcli roundtrips
// TODO: read wakatime config
// TODO: do not log when out of dev folder

#![expect(clippy::wildcard_imports, reason = "ls_types has no prelude")]
use ls_types::*;
use ls_types::{notification::Notification as _, request::Request as _};
use lsp_server::{Connection, ExtractError, Message, Notification, Request, RequestId};

// TODO: support independent backends
/// Open the Wakatime web dashboard in a browser
const OPEN_DASHBOARD_ACTION: &str = "Open wakatime.com dashboard";
/// Log the time past today in an editor
const SHOW_TIME_PAST_ACTION: &str = "Show time logged today";

/// Base plugin user agent
const PLUGIN_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct LanguageServer {
	connection: Connection,

	user_agent: String,
}

impl LanguageServer {
	#[must_use]
	pub fn new(connection: Connection) -> Self {
		Self {
			connection,
			user_agent: PLUGIN_USER_AGENT.into(),
		}
	}

	fn capabilities() -> ServerCapabilities {
		ServerCapabilities {
			text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::NONE)),
			execute_command_provider: Some(ExecuteCommandOptions {
				commands: vec![OPEN_DASHBOARD_ACTION.into(), SHOW_TIME_PAST_ACTION.into()],
				work_done_progress_options: WorkDoneProgressOptions::default(),
			}),
			..Default::default()
		}
	}

	/// Entrypoint
	///
	/// # Errors
	///
	/// - For kindof everything that went wrong
	pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
		let server_capabilities = serde_json::to_value(Self::capabilities())?;
		let init_params = self.connection.initialize(server_capabilities)?;
		let init_params = serde_json::from_value::<InitializeParams>(init_params)?;

		if let Some(info) = &init_params.client_info {
			self.user_agent = format!(
				"{}/{} {} {}-wakatime/{}",
				// Editor part
				info.name,
				info.version
					.as_ref()
					.map_or_else(|| "unknown", |version| version.as_str()),
				// Plugin part
				self.user_agent,
				// Last part is the one parsed by `wakatime` servers
				// It follows `{editor}-wakatime/{version}` where `editor` is
				// registered in intern. Works when `info.name` matches what the
				// wakatime dev choose.
				// IDEA: rely less on luck
				info.name,
				env!("CARGO_PKG_VERSION"),
			);
		}

		self.main_loop()?;

		Ok(())
	}

	fn main_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
		for msg in &self.connection.receiver {
			match msg {
				Message::Request(req) => {
					if self.connection.handle_shutdown(&req)? {
						return Ok(());
					}

					self.handle_request(req)?;
				}

				Message::Notification(notification) => {
					self.handle_notification(notification)?;
				}

				Message::Response(response) => {
					eprintln!("{response:?}");
				}
			}
		}

		Ok(())
	}

	fn handle_request(&self, req: Request) -> Result<(), Box<dyn std::error::Error>> {
		let req = match try_cast_r::<request::ExecuteCommand>(req)? {
			Ok((id, params)) => {
				self.execute_command(id, &params)?;
				return Ok(());
			}
			Err(req) => req,
		};

		let _ = req;

		Ok(())
	}

	fn handle_notification(
		&self,
		notification: Notification,
	) -> Result<(), Box<dyn std::error::Error>> {
		let notification = match try_cast_n::<notification::DidOpenTextDocument>(notification)? {
			Ok(params) => {
				self.on_change(&params.text_document.uri, false)?;
				return Ok(());
			}
			Err(req) => req,
		};

		let notification = match try_cast_n::<notification::DidChangeTextDocument>(notification)? {
			Ok(params) => {
				self.on_change(&params.text_document.uri, false)?;
				return Ok(());
			}
			Err(req) => req,
		};

		let notification = match try_cast_n::<notification::DidCloseTextDocument>(notification)? {
			Ok(params) => {
				self.on_change(&params.text_document.uri, false)?;
				return Ok(());
			}
			Err(req) => req,
		};

		let notification = match try_cast_n::<notification::DidSaveTextDocument>(notification)? {
			Ok(params) => {
				self.on_change(&params.text_document.uri, true)?;
				return Ok(());
			}
			Err(req) => req,
		};

		let _ = notification;

		Ok(())
	}

	fn on_change(&self, uri: &Uri, is_write: bool) -> Result<(), Box<dyn std::error::Error>> {
		let mut cmd = std::process::Command::new("wakatime-cli");

		cmd.args(["--plugin", &self.user_agent]);

		cmd.args(["--entity", uri.path().as_str()]);

		// cmd.args(["--lineno", ""]);
		// cmd.args(["--cursorno", ""]);
		// cmd.args(["--lines-in-file", ""]);
		// cmd.args(["--category", ""]);

		// cmd.args(["--alternate-project", ""]);
		// cmd.args(["--project-folder", ""]);

		if is_write {
			cmd.arg("--write");
		}

		let status = cmd.status()?;

		// error codes are available at
		// https://github.com/wakatime/wakatime-cli/blob/develop/pkg/exitcode/exitcode.go

		if !status.success() {
			let notification = Message::Notification(Notification::new(
				notification::ShowMessage::METHOD.into(),
				ShowMessageParams {
					typ: MessageType::WARNING,
					message: format!(
						"`wakatime-cli` exited with error code: {}. Check your configuration.",
						status
							.code()
							.map_or_else(|| "<none>".into(), |c| c.to_string())
					),
				},
			));
			self.connection.sender.send(notification)?;
		}

		Ok(())
	}

	fn execute_command(
		&self,
		id: RequestId,
		params: &ExecuteCommandParams,
	) -> Result<(), Box<dyn std::error::Error>> {
		match params.command.as_str() {
			OPEN_DASHBOARD_ACTION => {
				let show_documents_params = ShowDocumentParams {
					uri: "https://wakatime.com/dashboard"
						.parse()
						.expect("url is valid"),
					external: Some(true),
					take_focus: None,
					selection: None,
				};

				let req = Message::Request(Request::new(
					id,
					request::ShowDocument::METHOD.into(),
					show_documents_params,
				));
				self.connection.sender.send(req)?;
			}
			SHOW_TIME_PAST_ACTION => {
				let output = std::process::Command::new("wakatime-cli")
					.arg("--today")
					.output()?;

				let time_past = String::from_utf8_lossy(&output.stdout);

				let notification = Message::Notification(Notification::new(
					notification::ShowMessage::METHOD.into(),
					ShowMessageParams {
						typ: MessageType::INFO,
						message: time_past.to_string(),
					},
				));
				self.connection.sender.send(notification)?;
			}
			unknown_cmd_id => {
				let message = format!("Unknown workspace command received: `{unknown_cmd_id}`");

				let notification = Message::Notification(Notification::new(
					notification::ShowMessage::METHOD.into(),
					ShowMessageParams {
						typ: MessageType::ERROR,
						message,
					},
				));
				self.connection.sender.send(notification)?;
			}
		}

		Ok(())
	}
}

// first result if for json decoding error, second is for method mismatch
type CastResult<Payload, Type> = Result<Result<Payload, Type>, ExtractError<Type>>;

fn try_cast_r<R>(req: Request) -> CastResult<(RequestId, R::Params), Request>
where
	R: ls_types::request::Request,
	R::Params: serde::de::DeserializeOwned,
{
	match req.extract(R::METHOD) {
		Ok(params) => Ok(Ok(params)),
		Err(ExtractError::MethodMismatch(req)) => Ok(Err(req)),
		Err(err) => Err(err),
	}
}

fn try_cast_n<N>(notif: Notification) -> CastResult<N::Params, Notification>
where
	N: ls_types::notification::Notification,
	N::Params: serde::de::DeserializeOwned,
{
	match notif.extract(N::METHOD) {
		Ok(params) => Ok(Ok(params)),
		Err(ExtractError::MethodMismatch(notif)) => Ok(Err(notif)),
		Err(err) => Err(err),
	}
}
