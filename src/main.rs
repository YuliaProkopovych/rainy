use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use reqwest::header::USER_AGENT;
use std::collections::HashMap;


// This struct represents state
struct AppState {
  coordinates: (f32, f32),
}

#[get("/")]
async fn hello() -> impl Responder {
  let response = get_forecast().await.unwrap();

    HttpResponse::Ok().body(response)
}

async fn get_forecast() -> Result<String, Box<dyn std::error::Error>> {
  let client = reqwest::Client::new();
  let response = client.get("https://api.met.no/weatherapi/locationforecast/2.0/compact?lat=34&lon=65") // <- Create request builder
  .header(USER_AGENT, "https://github.com/YuliaProkopovych/weather-forecast")
  .send()
  .await
  .expect("failed to get response")
  .text()
  .await
  .expect("failed to get payload");
  println!("{:#?}", response);
  println!("Response: {:?}", "response");
  Result::Ok(response)
}

#[post("/forecast")]
async fn forecast(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

// fastify.register(require('./routes/forecast'));
// fastify.register(require('./routes/solar-forecast'));
// fastify.register(require('./routes/nearby-locations'));

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
