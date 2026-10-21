use chrono::Local;
use clap::Parser;
use gshock_api::{Connection, GshockAPI, WatchButton};

#[derive(Parser, Debug)]
#[command(author, version, about = "G-Shock Time Sync Server", long_about = None)]
struct Cli {
    /// Fine adjustment in seconds to add/subtract when setting time (-10 to 10)
    #[arg(short, long, default_value_t = 0)]
    fine_adjustment_secs: i32,

    /// Sets log level
    #[arg(short, long, default_value = "INFO")]
    log_level: String,
}

fn prompt() {
    println!("==============================================================================================");
    println!("Short-press lower-right button on your watch to set time...");
    println!();
    println!("If Auto-time set on watch, the watch will connect and run automatically up to 4 times per day.");
    println!("==============================================================================================");
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    println!("Starting G-Shock Time Server (fine adjustment: {}s)...", cli.fine_adjustment_secs);
    prompt();

    loop {
        println!("Waiting for connection...");
        let mut connection = Connection::new();

        if connection.connect().await? {
            println!("Connected...");
            let mut api = GshockAPI::new(&connection);

            let button = api.get_pressed_button().await.unwrap_or(WatchButton::NoButton);
            if !matches!(button, WatchButton::LowerRight | WatchButton::NoButton | WatchButton::LowerLeft) {
                println!("Button {:?} pressed, skipping time set...", button);
                let _ = connection.disconnect().await;
                continue;
            }

            let watch_name = api.get_watch_name().await.unwrap_or_default();
            println!("name: {}", watch_name);

            let fine_adjustment_secs = cli.fine_adjustment_secs as i64;
            if let Err(e) = api.set_system_time(fine_adjustment_secs).await {
                println!("Failed to set time: {}", e);
            } else {
                println!(
                    "Time set at {} on {}",
                    Local::now().format("%Y-%m-%d %H:%M:%S"),
                    watch_name
                );
            }

            if !api.model_info().always_connected {
                let _ = connection.disconnect().await;
            }
        }
    }
}
