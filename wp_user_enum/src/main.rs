use clap::Parser;
use regex::Regex;
use reqwest::blocking::Client;
use std::{
    collections::HashSet,
    thread::sleep,
    time::Duration,
};

/// WordPress user enumeration tool for internal audits
#[derive(Parser, Debug)]
#[command(
    name = "wp_user_digger",
    version = "0.1.0",
    about = "Enumerate WordPress users via author redirect (internal audit tool)"
)]
struct Args {
    /// Target WordPress base URL (e.g. https://example.com or https://example.com/wp)
    #[arg(long, value_name = "URL", required = true)]
    url: String,

    /// Start author ID
    #[arg(long, default_value_t = 1)]
    start: u32,

    /// End author ID
    #[arg(long, default_value_t = 100)]
    end: u32,

    /// Sleep seconds between requests (Fail2Ban/WAF avoidance)
    #[arg(long, default_value_t = 1)]
    sleep_sec: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Logical validation
    if args.start > args.end {
        eprintln!("error: --start must be less than or equal to --end");
        std::process::exit(2);
    }

    let base_url = args.url.trim_end_matches('/');

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("wp_user_digger/1.0 (internal security audit)")
        .build()?;

    // Match /author/<username>/ or /author/<username>
    let re = Regex::new(r"/author/([^/]+)/?")?;

    let mut users: HashSet<String> = HashSet::new();

    for author_id in args.start..=args.end {
        let target = format!("{}/?author={}", base_url, author_id);

        match client.get(&target).send() {
            Ok(resp) => {
                if resp.status().is_redirection() {
                    if let Some(location) = resp.headers().get("Location") {
                        if let Ok(loc_str) = location.to_str() {
                            if let Some(caps) = re.captures(loc_str) {
                                users.insert(caps[1].to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => {
                // 通信エラーは黙って継続（監査ツール向け挙動）
            }
        }

        sleep(Duration::from_secs(args.sleep_sec));
    }

    for user in users {
        println!("{}", user);
    }

    Ok(())
}

