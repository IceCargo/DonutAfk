use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use azalea::prelude::*;
use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, BufReader};

const SERVER: &str = "donutsmp.net"; //change ip how u want
const ACCOUNT: &str = "donutafk";
const JUMP_INTERVAL_TICKS: u32 = 20 * 45;

#[tokio::main]
async fn main() -> AppExit {
    let account = match Account::microsoft(ACCOUNT).await {
        Ok(acc) => acc,
        Err(err) => {
            eprintln!("login failed: {err}");
            std::process::exit(1);
        }
    };

    println!("logged in as '{}'", account.username());

    ClientBuilder::new()
        .set_handler(handle)
        .start(account, SERVER)
        .await
}

#[derive(Default, Clone, Component)]
pub struct State {
    ticks_since_jump: Arc<Mutex<u32>>,
    stdin_started: Arc<AtomicBool>,
}

async fn handle(bot: Client, event: Event, state: State) -> eyre::Result<()> {
    match event {
        Event::Spawn => {
            println!("connected on {SERVER}");

            if !state.stdin_started.swap(true, Ordering::SeqCst) {
                let bot = bot.clone();
                tokio::task::spawn_local(async move {
                    let mut lines = BufReader::new(tokio::io::stdin()).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let line = line.trim();
                        if !line.is_empty() {
                            bot.chat(line);
                        }
                    }
                });
            }
        }
        Event::Tick => {
            let mut ticks = state.ticks_since_jump.lock();
            *ticks += 1;
            if *ticks >= JUMP_INTERVAL_TICKS {
                *ticks = 0;
                bot.jump();
            }
        }
        Event::Chat(m) => {
            if let (None, content) = m.split_sender_and_content() {
                println!("{content}");
            }
        }
        Event::Disconnect(reason) => {
            let reason = reason
                .map(|r| r.to_ansi())
                .unwrap_or_else(|| "unknown".to_string());
            println!("disconnected bc: {reason}");
        }
        _ => {}
    }

    Ok(())
}