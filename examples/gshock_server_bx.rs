use chrono::{Datelike, Local, Timelike};
use gshock_api::{Connection, DateTimeParts, GshockAPI, WatchButton};

fn prompt() {
    println!("==============================================================================================");
    println!("GW-BX5600 / GMW-BZ5000 MIP Display Watch Time Server");
    println!("Short-press lower-right button on your watch to set time...");
    println!("==============================================================================================");
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    prompt();

    loop {
        println!("Waiting for GW-BX5600 watch connection...");
        let mut connection = Connection::new();
        if connection.connect().await? {
            println!("Connected to MIP display watch!");
            let mut api = GshockAPI::new(&connection);
            api.set_model_name("CASIO GW-BX5600");

            let button = api.get_pressed_button().await.unwrap_or(WatchButton::NoButton);
            if matches!(button, WatchButton::LowerRight | WatchButton::NoButton | WatchButton::LowerLeft) {
                let name = api.get_watch_name().await.unwrap_or_default();
                println!("Watch name: {}", name);

                let now = Local::now();
                let dt = DateTimeParts {
                    year: now.year() as u16,
                    month: now.month() as u8,
                    day: now.day() as u8,
                    hour: now.hour() as u8,
                    minute: now.minute() as u8,
                    second: now.second() as u8,
                    weekday: now.weekday().num_days_from_monday() as u8,
                    nanosecond: now.nanosecond(),
                };

                println!("Running GW-BX5600 time sequence for {:?}", dt);
                api.set_time(dt).await?;
                println!("GW-BX5600 time sequence complete at {}", now.format("%Y-%m-%d %H:%M:%S"));
            }

            connection.disconnect().await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
