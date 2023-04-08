use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::header::USER_AGENT;
use std::collections::HashMap;
use serde_json;
use serde_derive::{ Deserialize, Serialize };
use std::env;

// This struct represents state
struct AppState {
  coordinates: (f32, f32),
}

#[derive(Debug, Serialize, Deserialize)]
struct ForecastRecord {
  time: String,
  weather: HashMap<String, f32>,
  next_6_hours: Option<NextForecastRecord>,
  next_hour: Option<NextForecastRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
struct NextForecastRecord {
  precipitations: f64,
  symbol: String
}

#[derive(Debug, Serialize, Deserialize)]
struct Coordinates {
  lat: f64,
  lon: f64
}

#[get("/")]
async fn hello(coordinates: web::Query<Coordinates>) -> impl Responder {
  HttpResponse::Ok().body(String::from("hi"))
}

async fn get_forecast(coordinates: &Coordinates) -> Result<Vec<ForecastRecord>, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let addr = format!("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={0}&lon={1}", coordinates.lat, coordinates.lon);
  let response = client.get(addr)
  .header(USER_AGENT, "https://github.com/YuliaProkopovych/weather-forecast")
  .send()
  .await?;

  let res_json = response.json::<serde_json::Value>().await?;
  let properties = &res_json["properties"];
  let timeseries = &properties["timeseries"];
  let array = timeseries.as_array().expect("wowowow");
  let formatted_vec: Vec<ForecastRecord> = format_forecast((*array).clone());

  //println!("{:#?}", formatted_vec[0].next_hour.as_ref().unwrap());

  Result::Ok(formatted_vec)
}

#[derive(Debug, Serialize, Deserialize)]
struct TimezoneInfo {
  timezoneId: String,
  offset: f32
}

async fn get_timezones(coordinates: &Coordinates) -> Result<TimezoneInfo, Box<dyn std::error::Error>> {

  let client = reqwest::Client::new();
  let url = format!("https://www.timeapi.io/api/TimeZone/coordinate?latitude={0}&longitude={1}", coordinates.lat, coordinates.lon);
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

fn format_forecast(array: Vec<serde_json::Value>) -> Vec<ForecastRecord> {
  let mut formatted_vec: Vec<ForecastRecord> = vec![];
  for i in array {

    let data = &i["data"];
    let instant = &data["instant"];
    let details = &instant["details"];
    let next_6_hours = &data["next_6_hours"];
    let next_hour = &data["next_1_hours"];

    let mut rec = ForecastRecord {
      time: i["time"].as_str().unwrap().to_string(),
      weather: serde_json::from_value((*details).clone()).unwrap(),
      next_6_hours: None,
      next_hour: None,
    };

    if !next_6_hours.is_null() {
      let summary = &next_6_hours["summary"];
      let details = &next_6_hours["details"];
      rec.next_6_hours = Some(NextForecastRecord {
        precipitations: details["precipitation_amount"].as_f64().unwrap(),
        symbol: summary["symbol_code"].as_str().unwrap().to_string(),
      });
    }

    if !next_hour.is_null() {
      let summary = &next_hour["summary"];
      let details = &next_hour["details"];
      rec.next_hour = Some(NextForecastRecord {
        precipitations: details["precipitation_amount"].as_f64().unwrap(),
        symbol: summary["symbol_code"].as_str().unwrap().to_string(),
      });
    }

    formatted_vec.push(rec);
  }

  formatted_vec
}
#[derive(Debug, Serialize, Deserialize)]
struct ForecastInfo {
  forecast: Vec<ForecastRecord>,
  timezoneId: String,
  offset: f32,
  coordinates: Coordinates,
}

#[post("/forecast")]
async fn forecast(coordinates: web::Json<Coordinates>) -> impl Responder {
  //println!("Status: {:#?}", coordinates);
  let coords = coordinates.into_inner();
  let forecast = get_forecast(&coords).await.expect("fdsodfsd");
  let timezone = get_timezones(&coords).await.expect("sdfsdfsd");
  println!("Status: {:#?}", timezone);
  let response = ForecastInfo {
    forecast,
    coordinates: coords,
    timezoneId: timezone.timezoneId,
    offset: timezone.offset
  };
  let res_json = serde_json::to_string(&response).unwrap();

  HttpResponse::Ok().body(res_json)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// fastify.register(require('./routes/forecast'));
// fastify.register(require('./routes/solar-forecast'));
// fastify.register(require('./routes/nearby-locations'));

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  env::set_var("RUST_BACKTRACE", "1");
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(forecast)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
