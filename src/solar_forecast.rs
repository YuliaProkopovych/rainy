use actix_web::{get, web, HttpResponse, Responder};
use reqwest::header::USER_AGENT;
use serde_json;
use serde_derive::{ Deserialize, Serialize };
use chrono::prelude::*;

use crate::timezones::{ Coordinates, get_timezones };

#[derive(Deserialize, Debug)]
pub struct Info {
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
pub async fn solar_forecast(info: web::Query<Info>) -> impl Responder {
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
