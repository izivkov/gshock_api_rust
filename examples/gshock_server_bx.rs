use chrono::Local;
use gshock_api::{Connection, GshockAPI, WatchButton};

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

                println!("Running GW-BX5600 time sequence...");
                api.set_system_time(0).await?;
                println!("GW-BX5600 time sequence complete at {}", Local::now().format("%Y-%m-%d %H:%M:%S"));
            }

            connection.disconnect().await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}
