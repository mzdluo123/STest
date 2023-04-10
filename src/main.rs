use std::error::Error;
use std::io::ErrorKind;
use std::io::ErrorKind::BrokenPipe;
use chrono::Local;
use hyper::body::HttpBody;
use hyper::{Body, Method, Request};


type SError<T> = Result<T, Box<dyn Error + Send + Sync>>;

static UNITS: [&str; 3] = ["B/s", "KB/s", "MB/s"];

fn show_speed(speed_list: &[f32]) -> String {
    let mut speed: f32 = speed_list.iter().sum();
    let mut unit = 0;
    while speed > 1000.0 && unit < UNITS.len() - 1 {
        speed /= 1024.0;
        unit += 1;
    }
    return format!("{}{}", speed, UNITS[unit]);
}

#[derive(Debug)]
struct TestResult {
    len: usize,
    time: i64,
}

impl TestResult {
    fn to_speed(self) -> f32 {
        (self.len as f32) / (self.time as f32) * 1000.0  //每秒钟速度
    }
}


fn now() -> i64 {
    let dt = Local::now();
    return dt.timestamp_millis();
}


#[tokio::main]
async fn main() {
    let results = tokio::join!(
         test_download("http://dldir1.qq.com/qqfile/qq/QQNT/5333e29d/QQ_v6.9.12-10951.dmg"),
         test_download("http://dtapp-pub.dingtalk.com/dingtalk-desktop/mac_dmg/Release/DingTalk_v7.0.20.13_29200617_x86.dmg"),
        test_download("http://issuepcdn.baidupcs.com/issue/netdisk/yunguanjia/BaiduNetdisk_7.26.0.10.exe")
    );
    let r = [results.0.unwrap().to_speed(), results.1.unwrap().to_speed(), results.2.unwrap().to_speed()];
    println!("{}", show_speed(&r))
}


async fn test_download(url: &str) -> SError<TestResult> {
    let mut downloaded = 0usize;
    let client = hyper::Client::new();
    let req = Request::builder()
        .header("Range", "bytes=0-200000000")
        .uri(url)
        .method(Method::GET)
        .body(Body::empty())?;
    let mut rsp = client.request(req).await?;
    if rsp.status() == 206 {
        let start_time = now();
        while let Some(chunk) = rsp.body_mut().data().await {
            let c_len = chunk?.len();
            downloaded += c_len;
            if now() - start_time > 5_000 { // 最多下载十秒钟
                return Ok(TestResult {
                    len: downloaded,
                    time: now() - start_time,
                });
            }
        }
        return Ok(TestResult {
            len: downloaded,
            time: now() - start_time,
        });
    }

    Err(std::io::Error::from(BrokenPipe).into())
}