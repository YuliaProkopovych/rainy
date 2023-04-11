use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::header::USER_AGENT;
use std::collections::HashMap;
use serde_json;
use serde_derive::{ Deserialize, Serialize };
use std::env;
use chrono::prelude::*;
use chrono::offset::LocalResult;

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
  let coords = coordinates.into_inner();
  let forecast = get_forecast(&coords).await.expect("fdsodfsd");
  let timezone = get_timezones(&coords).await.expect("sdfsdfsd");

  let response = ForecastInfo {
    forecast,
    coordinates: coords,
    timezoneId: timezone.timezoneId,
    offset: timezone.offset
  };
  let res_json = serde_json::to_string(&response).unwrap();

  HttpResponse::Ok().body(res_json)
}

#[derive(Deserialize, Debug)]
struct Info {
    coordinates: String,
    startDate: String,
    endDate: String,
}

async fn get_sunrise(coords:&Coordinates, start_date: String, end_date: String, offset_string: String) -> Result<String, Box<dyn std::error::Error>> {

  let start = start_date.clone() + " 00:00:00";
  let end = end_date + " 00:00:00";

  let start = Utc.datetime_from_str(&start, "%Y-%m-%d %H:%M:%S").unwrap();
  let end = Utc.datetime_from_str(&end, "%Y-%m-%d %H:%M:%S").unwrap();

  let diff = end - start;

  let client = reqwest::Client::new();
  let url = format!("https://api.met.no/weatherapi/sunrise/2.0/.json?lat={0}&lon={1}&date={2}&offset={3}&days={4}",
    coords.lat, coords.lon, start_date, offset_string, diff.num_days());
  let response = client.get(url)
    .header(USER_AGENT, "https://github.com/YuliaProkopovych/weather-forecast")
    .send()
    .await?;

    let res_json = response.json::<serde_json::Value>().await?;
    let data = &res_json["location"];
    let array = &data["time"];
    println!("{:?}", data );


  // const response = await request;
  // const data = response.data.location.time;
  // data.pop(); //????

    Result::Ok(array.clone().to_string())

}

#[derive(Serialize, Debug)]
struct SolarData {
    coordinates: Coordinates,
    solarData: String,
    timezone: String,
}

#[get("/solar-forecast")]
async fn solar_forecast(info: web::Query<Info>) -> impl Responder {
  let parts: Vec<&str> = info.coordinates.split(",").collect();
  let coords = Coordinates {
    lat: parts[0].parse::<f64>().unwrap(),
    lon: parts[1].parse::<f64>().unwrap(),
  };

  let timezone = get_timezones(&coords).await.expect("sdfsdfsd");
  let offset = timezone.offset;
  let mut offset_string = format!( "{0}:{1}", ((offset.trunc()).abs()),(60.0 * (offset.fract())));
    if offset.abs() < 10.0 {
      offset_string = format!("0{}", offset_string);
    }
    if offset.fract() == 0.0 {
      offset_string = format!("{}0", offset_string);
    }
    if offset >= 0.0 {
      offset_string = format!("+{}", offset_string);
    } else {
      offset_string = format!("-{}", offset_string);
    }


  let solar_data = get_sunrise(&coords, info.startDate.clone(), info.endDate.clone(), offset_string).await.unwrap();
  println!("{:?}", solar_data );
  let response = SolarData {
    coordinates: coords,
    timezone: timezone.timezoneId,
    solarData: solar_data
  };
  let res_json = serde_json::to_string(&response).unwrap();

  //   reply.send({ solarData, coordinates, timezone });

  HttpResponse::Ok().body(res_json)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(forecast)
            .service(solar_forecast)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
