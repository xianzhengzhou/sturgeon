use log::{info};
use clap::{App, Arg, value_t};
use tokio::time::{delay_for};
use std::{thread, time, error};
use hyper::service::{make_service_fn, service_fn};
use std::convert::Infallible;
use hyper::{Body, Request, Response, Server};

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
  Ok(Response::new(Body::from("Hello World!")))
}

async fn long_op(id: &str) {
  tokio::task::spawn_blocking(move || {
    thread::sleep(time::Duration::from_secs(2));
  }).await;
  info!("{} has run.", id);
}

async fn start_fixed_job(id: &'static str, _interval: u64) {
  loop {
    delay_for(time::Duration::from_secs(_interval)).await;
    tokio::task::spawn(long_op(id));
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
  let _matches = App::new("sturgeon")
    .about("data scrapping program")
    .version("0.1.0")
    .author("Xianzheng Zhou <xianzheng.zhou@gmail.com>")
    .arg(
      Arg::with_name("type")
        .help("type of the scrapping [fixed|cron]")
        .short("t")
        .long("type")
        .takes_value(true)
        .required(true),
    )
    .arg(
      Arg::with_name("fixed")
        .help("fixed interval")
        .short("x")
        .long("fixed")
        .takes_value(true)
        .required_if("type", "fixed"),
    )
    .arg(
      Arg::with_name("cron")
        .help("cron expression")
        .short("c")
        .long("cron")
        .takes_value(true)
        .required_if("type", "cron"),
    )
    .get_matches();
  simple_logger::init().unwrap();

  let fixed_job = match _matches.value_of("type") {
    Some("fixed") => match value_t!(_matches, "fixed", u64) {
      Ok(_interval) => tokio::spawn(start_fixed_job("apple", _interval)),
      Err(e) => e.exit(),
    },
    Some(cmd) => panic!("Invalid type '{}' specified.", cmd),
    _ => panic!("Invalid character in type"),
  };
  let test_job = tokio::spawn(start_fixed_job("banana", 2));
  
  let make_svc = make_service_fn(|_conn| {
    // This is the `Service` that will handle the connection.
    // `service_fn` is a helper to convert a function that
    // returns a Response into a `Service`.
    async { Ok::<_, Infallible>(service_fn(hello)) }
  });

  let addr = ([127, 0, 0, 1], 3000).into();

  let server = Server::bind(&addr).serve(make_svc);

  println!("Listening on http://{}", addr);

  server.await?;

  Ok(())
}
