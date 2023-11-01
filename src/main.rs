use chrono::prelude::*;
use reqwest::Error;

#[macro_use]
extern crate prettytable;
use prettytable::Table;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let response_ipinfo = reqwest::get("https://ipinfo.io/ip").await?;
    let public_ip = response_ipinfo.text().await?;
    let info = geolocation::find(public_ip.as_str()).unwrap();
    // Tomorrow.io API endpoint URL
    let api = format!(
        "https://api.tomorrow.io/v4/timelines?location=
        {},
        {}
        &fields=temperature&timesteps=1h&units=metric&apikey=6IpzeOCbBDCr3ei3hnO7L3hGTVMRVKWg",
        info.longitude, info.latitude
    );

    // Make a GET request to the Tomorrow.io API endpoint
    let response = reqwest::get(api).await?;

    let mut table = Table::new();

    table.add_row(row!["Time", c->"Temperature"]);
    // Check if the request was successful (status code 200)
    if response.status().is_success() {
        // Parse and filter the temperature values for the entire day from the API response (JSON data)
        let json_response = response.json::<serde_json::Value>().await?;

        // Get current date
        let current_date = Utc::now().date_naive();

        // Extract and print hourly temperature values for the entire day
        if let Some(data) = json_response["data"]["timelines"][0]["intervals"].as_array() {
            for interval in data {
                if let Some(timestamp) = interval["startTime"].as_str() {
                    // Parse the timestamp and check if it's within the current day
                    if let Ok(parsed_timestamp) = timestamp.parse::<DateTime<Utc>>() {
                        if parsed_timestamp.date_naive() == current_date {
                            // Extract the hour component of the timestamp
                            let hour = parsed_timestamp.time().hour();
                            if let Some(temperature) = interval["values"]["temperature"].as_f64() {
                                // Add hour and temperature to the table
                                table.add_row(row![
                                    format!("{}:00", hour),
                                    c->format!("{:.2}°C", temperature)
                                ]);
                            }
                        }
                    }
                }
            }
        } else {
            println!("Temperature data not available in the API response.");
        }
    } else {
        // Handle API errors
        println!("Error: {}", response.status());
    }

    table.printstd();
    Ok(())
}
