#![allow(non_snake_case)]

use std::{env};
use std::path::Path;
use hyper::{Client};
use hyper_tls::HttpsConnector;
use std::io::Cursor;

static mut DEBUG: bool = true; // Define global debug variable
static mut SKIPDEBUGCHECK: bool = false;

struct Logger;
impl Logger {
    #[allow(dead_code)]
    fn info(message:&str) {
        println!("[INFO] {}", message)
    }
    #[allow(dead_code)]
    fn infoAD(message:&str, additional:&str) {
        println!("[INFO] {} {}", message, additional)
    }
    #[allow(dead_code)]
    fn error(message:&str) {
        println!("[ERROR] {}", message)
    }
    #[allow(dead_code)]
    fn errorAD(message:&str, additional:&str) {
        println!("[INFO] {} {}", message, additional);
    }
    #[allow(dead_code)]
    fn critical(message:&str) {
        println!("[CRITICAL] {}", message)
    }
    #[allow(dead_code)]
    fn criticalAD(message:&str, additional:&str) {
        println!("[CRITICAL] {} {}", message, additional);
    }
    #[allow(dead_code)]
    fn debug(message:&str) {
        unsafe {
            if DEBUG {
                println!("[DEBUG] {}", message)
            }
        }
    }
    #[allow(dead_code)]
    fn debugARRAY(message:&str, additional:&Vec<String>) {
        unsafe {
            if DEBUG {
                println!("[DEBUG] {} {:?}", message, additional);
            }
        }
    }
    #[allow(dead_code)]
    fn debugAD(message:&str, additional:&str) {
        unsafe {
            if DEBUG {
                println!("[DEBUG] {} {}", message, additional);
            }
        }
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

async fn fetch_url(url: String, file_name: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    unsafe {
        if DEBUG {
            Logger::debugAD("MP4 Response Code: {}", response.status().as_str())
        }
    }
    let mut file = std::fs::File::create(file_name)?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

fn autoDetectDebug() -> bool {
    unsafe {
        if SKIPDEBUGCHECK {
            return false;
        }
    }
    Path::new("Cargo.toml").exists()
}

#[tokio::main]
async fn main() -> Result<()>{
    unsafe {
        let args: Vec<String> = env::args().collect();
        let mut url : &str = "";
        let mut found: bool = false;
        DEBUG = autoDetectDebug();
        if args.len()==1  {
            println!("Voe-DL Rust Edition => !Not enough arguments!");
            std::process::exit(0)
        }
        if args.len()==2 {
            url = &args[1];
            found = true;
        }else {
            if args.len()==3  {
                if args[1].eq("--debug") {
                    DEBUG = true;
                    url = &args[2];
                    found = true;
                }
            }
        }
        Logger::debug("Debug active");
        Logger::debugARRAY("Detected arguments: ", &args);
        if found {
            Logger::debugAD("Try to download: ", url);
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let res = client.get(url.parse::<hyper::Uri>().unwrap()).await?;
            let body_bytes = hyper::body::to_bytes(res.into_body()).await?;
            let cache: String = String::from_utf8(body_bytes.to_vec()).unwrap();
            Logger::debugAD("Website responded with: ", &cache);
            let mut mp4url: String = "".to_string();
            let mut title: String = String::from("");
            for s in cache.split("\n") {
                if s.contains("<title>") {
                    title = String::from(s.replace(" - VOE | Content Delivery Network (CDN) & Video Cloud", "").replace("<title>", "").replace("</title>", ""))
                }
                if s.contains("'mp4'") {
                    mp4url = s.replace("'mp4': '", "").replace("'", "'").replace("',", "").replace("            ", "");
                }
            }
            Logger::debugAD("Detected Title: ", &title.replace(".mp4", ""));
            Logger::debugAD("VideoURL: ", &mp4url);
            Logger::info("Downloading...");
            fetch_url(mp4url, title.to_string()).await.unwrap();
            Logger::info("Download Complete!")
        }
        Ok(())
    }
}