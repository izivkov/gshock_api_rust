use chrono::{Datelike, Local, Timelike};
use gshock_api::{
    Alarm, AppNotification, Connection, DateTimeParts, GshockAPI, NotificationType,
};

fn prompt() {
    println!("========================================================================");
    println!("Press and hold lower-left button on your watch for 3 seconds to start...");
    println!("========================================================================");
    println!();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    prompt();

    let mut connection = Connection::new();
    println!("Waiting for connection...");

    if connection.connect().await? {
        println!("Connected to watch!");
        let mut api = GshockAPI::new(&connection);

        let app_info = api.get_app_info().await?;
        println!("App Info: {}", app_info);

        let home_time = api.get_home_time().await?;
        println!("Home Time: {}", home_time);

        let button = api.get_pressed_button().await?;
        println!("Pressed Button: {:?}", button);

        let watch_name = api.get_watch_name().await?;
        println!("Watch Name: {}", watch_name);

        let condition = api.get_watch_condition().await?;
        println!("Watch Condition: {:?}", condition);

        let steps_today = api.get_step_count_today().await?;
        println!("Steps Today: {}", steps_today);

        // Time sync
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
        api.set_time(dt).await?;
        println!("Time synced successfully.");

        // Alarms
        let alarms = api.get_alarms().await?;
        println!("Alarms: {:?}", alarms);
        api.set_alarms(&[Alarm::new(7, 25, true, false)]).await?;
        println!("Alarms set successfully.");

        // Timer
        let timer = api.get_timer().await?;
        println!("Timer: {} seconds", timer);
        api.set_timer(timer + 10).await?;

        // Settings
        let settings = api.get_basic_settings().await?;
        println!("Basic Settings: {:?}", settings);

        // App Notification
        let notification = AppNotification::new(
            NotificationType::Calendar,
            "20260516T233000",
            "Calendar",
            "Meeting",
            "Discuss project status",
            "",
        );
        api.send_app_notification(&notification).await?;
        println!("App Notification sent.");

        connection.disconnect().await?;
        println!("--- END OF TESTS ---");
    } else {
        println!("Could not connect to watch before timeout.");
    }

    Ok(())
}
