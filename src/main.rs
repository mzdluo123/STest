use chrono::Local;
use reqwest::header::HeaderValue;
use reqwest::{Method, Request};
use std::error::Error;
use std::io::ErrorKind;


type SError<T> = Result<T, Box<dyn Error + Send + Sync>>;

static UNITS: [&str; 3] = ["B/s", "KB/s", "MB/s"];
static TEST_TIME: i64 = 8000;

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
    let results = futures::future::join_all(vec![
        tokio::spawn(test_download("http://pcclient.download.youku.com/iku-win-release/youkuclient_setup_9.2.15.1002.exe")),
        tokio::spawn(test_download("http://speedxbu.baidu.com/shurufa/ime/setup/BaiduPinyinSetup_5.9.2.1.exe")),
        tokio::spawn(test_download("http://x19.gdl.netease.com/MCLauncher_1.9.0.2363.exe")),
        tokio::spawn(test_download("http://dldir1.qq.com/qqfile/qq/QQNT/5333e29d/QQ_v6.9.12-10951.dmg")),
        tokio::spawn(test_download("http://dtapp-pub.dingtalk.com/dingtalk-desktop/mac_dmg/Release/DingTalk_v7.0.20.13_29200617_x86.dmg")),
        tokio::spawn(test_download("http://issuepcdn.baidupcs.com/issue/netdisk/yunguanjia/BaiduNetdisk_7.26.0.10.exe")),
        tokio::spawn(test_download("http://consumer.huawei.com/content/dam/huawei-cbg-site/cn/mkt/mobileservices/browser/exe/PCforX64.exe")),
    ]).await;

    let mut total_speed = 0f32;
    for i in results.into_iter().flatten().flatten() {
        total_speed += i.to_speed();
    }

    println!("{}", show_speed(total_speed))
}

async fn test_download(url: &str) -> SError<TestResult> {
    // println!("start test for {}", url);
    let mut downloaded = 0usize;
    let client = reqwest::Client::new();
    client.execute(Request::new(Method::HEAD, url.parse()?)).await?;
    let start_time = now();
    'timeout: while now() - start_time < TEST_TIME {  // 时间不到就一直下载数据
        let mut req = Request::new(Method::GET, url.parse()?);
        req.headers_mut()
            .append("range", HeaderValue::from_str("bytes=0-100000000")?);
        let mut rsp = client.execute(req).await?;
        if rsp.status() == 206 {
            while let Ok(chunk) = rsp.chunk().await {
                match chunk {
                    Some(c) => {
                        downloaded += c.len();
                        if now() - start_time > TEST_TIME {  // 时间到了就返回结果
                           break 'timeout;
                        }
                    }
                    None => break,
                }
            }
        } else {
            return Err(std::io::Error::from(ErrorKind::NotFound).into());
        }
    }
    Ok(TestResult {
        len: downloaded,
        time: now() - start_time,
    })
}
