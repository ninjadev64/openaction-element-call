use openaction::*;
use simplelog::{CombinedLogger, ConfigBuilder, LevelFilter, WriteLogger};
use tokio::{spawn, sync::Mutex};

use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

static SOCKETS: Mutex<Vec<SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>>> =
	Mutex::const_new(vec![]);
static ACTIONS: Mutex<Vec<(String, String)>> = Mutex::const_new(vec![]);

struct GlobalEventHandler {}
impl openaction::GlobalEventHandler for GlobalEventHandler {}

struct ActionEventHandler {}
impl openaction::ActionEventHandler for ActionEventHandler {
	async fn will_appear(
		&self,
		event: AppearEvent,
		_outbound: &mut OutboundEventManager,
	) -> EventHandlerResult {
		ACTIONS.lock().await.push((event.action, event.context));
		Ok(())
	}

	async fn will_disappear(
		&self,
		event: AppearEvent,
		_outbound: &mut OutboundEventManager,
	) -> EventHandlerResult {
		ACTIONS.lock().await.retain(|(_, c)| c != &event.context);
		Ok(())
	}

	async fn key_up(
		&self,
		event: KeyEvent,
		outbound: &mut openaction::OutboundEventManager,
	) -> EventHandlerResult {
		match &event.action[..] {
			"com.amansprojects.elementcall.togglemic" => {
				broadcast_message("toggle_mic", outbound).await
			}
			"com.amansprojects.elementcall.togglecamera" => {
				broadcast_message("toggle_camera", outbound).await
			}
			_ => Ok(()),
		}
	}
}

async fn broadcast_message(
	message: &str,
	outbound: &mut OutboundEventManager,
) -> EventHandlerResult {
	let mut sockets = SOCKETS.lock().await;
	if sockets.len() == 0 {
		let _ = outbound
			.open_url(
				"https://github.com/ninjadev64/openaction-element-call/blob/main/README.md"
					.to_owned(),
			)
			.await;
		return Ok(());
	}
	for socket in sockets.iter_mut() {
		let _ = socket.send(Message::text(message)).await;
	}
	Ok(())
}

async fn handle_message(message: Result<Message, tokio_tungstenite::tungstenite::Error>) {
	let Ok(message) = message else { return };
	if let Message::Text(s) = message {
		let actions = ACTIONS.lock().await;
		let mut outbound = OUTBOUND_EVENT_MANAGER.lock().await;
		let outbound = outbound.as_mut().unwrap();

		let set_action_to_state = |action: String, state: u16| async move {
			for (_, c) in actions.iter().filter(|(a, _)| a == &action) {
				let _ = outbound.set_state(c.clone(), state).await;
			}
		};

		match &s[..] {
			"mic_on" => {
				set_action_to_state("com.amansprojects.elementcall.togglemic".to_owned(), 0).await
			}
			"mic_off" => {
				set_action_to_state("com.amansprojects.elementcall.togglemic".to_owned(), 1).await
			}
			"camera_on" => {
				set_action_to_state("com.amansprojects.elementcall.togglecamera".to_owned(), 0)
					.await
			}
			"camera_off" => {
				set_action_to_state("com.amansprojects.elementcall.togglecamera".to_owned(), 1)
					.await
			}
			_ => (),
		}
	}
}

#[tokio::main]
async fn main() {
	CombinedLogger::init(vec![WriteLogger::new(
		LevelFilter::Debug,
		ConfigBuilder::new()
			.add_filter_ignore_str("tungstenite")
			.build(),
		std::fs::File::create("plugin.log").unwrap(),
	)])
	.unwrap();

	spawn(async {
		let server = match tokio::net::TcpListener::bind("0.0.0.0:57111").await {
			Ok(server) => server,
			Err(error) => {
				log::error!("{}", error);
				return;
			}
		};
		while let Ok((stream, _)) = server.accept().await {
			let Ok(ws) = tokio_tungstenite::accept_async(stream).await else {
				continue;
			};
			let (read, write) = ws.split();
			SOCKETS.lock().await.push(read);
			spawn(write.for_each(handle_message));
		}
	});

	if let Err(error) = init_plugin(GlobalEventHandler {}, ActionEventHandler {}).await {
		log::error!("Failed to initialise plugin: {}", error);
	}
}
