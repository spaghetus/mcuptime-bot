use std::time::{Duration, Instant};

use clap::Parser;
use craftping::{tokio::ping, Response};
use serenity::{prelude::GatewayIntents, Client};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
struct Args {
	#[arg(short, long, env = "DISCORD_TOKEN")]
	pub token: String,
	#[arg(short, long, env = "DISCORD_CHANNEL")]
	pub channel: u64,
	#[arg(short = 's', long, env = "MINECRAFT_HOST")]
	pub mc_server: String,
	#[arg(short = 'p', long, env = "MINECRAFT_PORT", default_value = "25565")]
	pub mc_port: u16,
	#[arg(short = 'i', long, env = "PING_INTERVAL", default_value = "5")]
	pub interval: u64,
}

#[tokio::main]
async fn main() {
	// Build client
	let args = Args::parse();
	let client = Client::builder(
		args.token,
		GatewayIntents::default() | GatewayIntents::GUILD_MESSAGES,
	)
	.await
	.expect("Failed to build client.");
	// Get handle for channel
	let channel = client
		.cache_and_http
		.http
		.get_channel(args.channel)
		.await
		.expect("Failed to get channel.");
	// Get current server info
	let server = args.mc_server;
	let port = args.mc_port;
	let mut info = ServerInfo::get(&server, port).await;
	println!("Enter main loop");
	let mut last_change = Instant::now();
	// Send hello message
	channel
		.id()
		.send_message(&*client.cache_and_http.http, |msg| {
			msg.content(format!(
				"MCUptime is ready! Server is {}.",
				if info.0.is_some() { "up" } else { "down" }
			))
		})
		.await
		.expect("Failed to send hello");
	// Start main loop
	loop {
		tokio::time::sleep(Duration::from_secs(args.interval * 60)).await;
		let new_info = ServerInfo::get(&server, port).await;
		if new_info.0.is_some() ^ info.0.is_some() {
			let now = Instant::now();
			channel
				.id()
				.send_message(&*client.cache_and_http.http, |msg| {
					msg.content(format!(
						"Server came {} after {} minutes since last change.",
						if info.0.is_some() { "up" } else { "down" },
						(now - last_change).as_secs() / 60,
					))
				})
				.await
				.expect("Failed to send status update");
			last_change = now;
		}
		info = new_info;
	}
}

struct ServerInfo(pub Option<Response>);

impl ServerInfo {
	async fn get(server: &str, port: u16) -> Self {
		if let Ok(mut stream) = TcpStream::connect((server, port)).await {
			if let Ok(Ok(ping)) =
				tokio::time::timeout(Duration::from_secs(10), ping(&mut stream, server, port)).await
			{
				ServerInfo(Some(ping))
			} else {
				ServerInfo(None)
			}
		} else {
			ServerInfo(None)
		}
	}
}
