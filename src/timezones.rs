use serde_json;
use serde_derive::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinates {
  pub lat: f64,
  pub lon: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimezoneInfo {
  pub timezoneId: String,
  pub offset: f32
}

pub async fn get_timezones(coords: &Coordinates) -> Result<TimezoneInfo, Box<dyn std::error::Error>> {

  let client = reqwest::Client::new();
  let url = format!("https://www.timeapi.io/api/TimeZone/coordinate?latitude={0}&longitude={1}", coords.lat, coords.lon);
  let response = client.get(url)
    .send()
    .await?;

  let res_json = response.json::<serde_json::Value>().await?;

  let current_offset = &res_json["currentUtcOffset"];
  let offset_in_seconds = &current_offset["seconds"];
  let mut current_offset: f32 = serde_json::from_value((*offset_in_seconds).clone()).unwrap();
  current_offset = current_offset / 3600.0;
  let timezone_id = res_json["timeZone"].as_str().unwrap().to_string();
  let result = TimezoneInfo {
    timezoneId: timezone_id,
    offset: current_offset
  };

  Result::Ok( result)
}
