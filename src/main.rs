use std::error::Error;
use std::ops::Add;
use chrono::Local;
use hyper::body::HttpBody;


type SError<T> = Result<T, Box<dyn Error>>;

static UNITS: [&str; 3] = ["B/s", "KB/s", "MB/s"];

#[derive(Debug)]
struct TestResult {
    len: usize,
    time: i64,
}

impl TestResult {
    fn to_speed(self) -> f32 {
        (self.len as f32) / (self.time as f32) * 1000.0  //每秒钟速度
    }
    fn show(self) -> String {
        let mut speed: f32 = self.to_speed().into();
        let mut unit = 0;
        while speed > 1000.0 && unit < UNITS.len() - 1 {
            speed /= 1000.0;
            unit += 1;
        }
        return format!("{}{}", speed, UNITS[unit]);
    }
}

impl Add for TestResult {
    type Output = TestResult;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            len: self.len + rhs.len,
            time: (self.time + rhs.time) / 2,
        }
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
        test_download("http://issuepcdn.baidupcs.com/issue/netdisk/MACguanjia/4.19.2/BaiduNetdisk_mac_4.19.2_x64.dmg")
    );
    let r = results.0.unwrap() + results.1.unwrap() + results.2.unwrap();
    println!("{:?}", r.show())
}


async fn test_download(url: &str) -> SError<TestResult> {
    let mut downloaded = 0usize;
    let client = hyper::Client::new();
    let url = url.parse()?;
    let mut rsp = client.get(url).await?;
    if rsp.status() == 200 {
        let start_time = now();
        while let Some(chunk) = rsp.body_mut().data().await {
            let c_len = chunk?.len();
            if c_len == 0usize {
                return Ok(TestResult {
                    len: downloaded,
                    time: now() - start_time,
                });
            }
            downloaded += c_len;
            if now() - start_time > 10_000 { // 最多下载十秒钟
                return Ok(TestResult {
                    len: downloaded,
                    time: now() - start_time,
                });
            }
        }
    }

    Ok(TestResult {
        len: 0,
        time: 0,
    })
}