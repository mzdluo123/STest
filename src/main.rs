use chrono::Local;
use reqwest::header::HeaderValue;
use reqwest::{Method, Request};
use std::error::Error;
use std::io::ErrorKind;


type SError<T> = Result<T, Box<dyn Error + Send + Sync>>;

static UNITS: [&str; 3] = ["B/s", "KB/s", "MB/s"];
//static TEST_TIME: i64 = 60*1000;

fn show_speed(mut speed: f32) -> String {
    let mut unit = 0;
    while speed > 1000.0 && unit < UNITS.len() - 1 {
        speed /= 1024.0;
        unit += 1;
    }
    format!("{}{}", speed, UNITS[unit])
}

#[derive(Debug)]
struct TestResult {
    len: usize,
    time: i64,
}

impl TestResult {
    fn to_speed(self) -> f32 {
        (self.len as f32) / (self.time as f32) * 1000.0 //每秒钟速度
    }
}

fn now() -> i64 {
    let dt = Local::now();
    dt.timestamp_millis()
}

#[tokio::main]
async fn main() {
    // 获取程序执行参数
    let args: Vec<String> = std::env::args().collect();
    let threads = if args.len() == 2 {
        args[1].parse::<usize>().unwrap_or(2)   
    }else{
        2
    };
    let mut tasks = vec![];
    let waste_url = "https://db.laomoe.com/data-waster-dummy";
    for _i in 0..threads {
        tasks.push(tokio::spawn(test_download(&waste_url)));
    }
    let _ = futures::future::join_all(tasks).await;
}

async fn test_download(url: &str) -> SError<TestResult> {
    // println!("start test for {}", url);
    let client = reqwest::Client::new();
    if let Err(e) = client.execute(Request::new(Method::HEAD, url.parse()?)).await{
        println!("download error:{} url:{}",e,url);
        return Err(e.into());
    }
    loop {
        let mut req = Request::new(Method::GET, url.parse()?);
        req.headers_mut()
            .append("range", HeaderValue::from_str("bytes=0-100000000")?);
        let mut rsp = client.execute(req).await.unwrap();
        // dbg!(rsp.status());
        if rsp.status() == 206|| rsp.status() == 200 {
            while let Ok(chunk) = rsp.chunk().await {
                if chunk.is_none(){
                    break;
                }
            }
        } 
    }
}
