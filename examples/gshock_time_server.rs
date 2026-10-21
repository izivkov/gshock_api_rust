use chrono::{Datelike, Local, Timelike};
use clap::Parser;
use gshock_api::{Connection, DateTimeParts, GshockAPI, WatchButton};

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
        println!("Waiting for Casio BLE watch connection...");
        let mut connection = Connection::new();
        match connection.connect().await {
            Ok(true) => {
                println!("Connected to watch!");
                let mut api = GshockAPI::new(&connection);

                let button = api.get_pressed_button().await.unwrap_or(WatchButton::NoButton);
                if !matches!(button, WatchButton::LowerRight | WatchButton::NoButton | WatchButton::LowerLeft) {
                    println!("Ignored press for button: {:?}", button);
                    let _ = connection.disconnect().await;
                    continue;
                }

                let watch_name = api.get_watch_name().await.unwrap_or_default();
                println!("Watch name: {}", watch_name);

                let now = Local::now();
                let adjusted_now = now + chrono::Duration::seconds(cli.fine_adjustment_secs as i64);

                let dt = DateTimeParts {
                    year: adjusted_now.year() as u16,
                    month: adjusted_now.month() as u8,
                    day: adjusted_now.day() as u8,
                    hour: adjusted_now.hour() as u8,
                    minute: adjusted_now.minute() as u8,
                    second: adjusted_now.second() as u8,
                    weekday: adjusted_now.weekday().num_days_from_monday() as u8,
                    nanosecond: adjusted_now.nanosecond(),
                };

                println!("Setting time to: {:?}", dt);
                if let Err(e) = api.set_time(dt).await {
                    println!("Failed to set time: {}", e);
                } else {
                    println!("Time successfully set at {}", adjusted_now.format("%Y-%m-%d %H:%M:%S"));
                }

                let _ = connection.disconnect().await;
            }
            Ok(false) => {
                println!("No Casio watch found in scan interval, retrying...");
            }
            Err(e) => {
                println!("Connection error: {}, retrying...", e);
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
