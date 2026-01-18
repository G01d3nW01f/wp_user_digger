use clap::Parser;
use regex::Regex;
use reqwest::blocking::Client;
use std::{
    collections::HashSet,
    thread::sleep,
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(
    name = "wp_user_digger",
    version = "0.3.0",
    about = "Enumerate WordPress usernames via author redirects and HTML inspection (internal audit tool)"
)]
struct Args {
    /// Target WordPress base URL (e.g. http://example.com or http://example.com/wordpress)
    #[arg(long, required = true)]
    url: String,

    /// Start author ID
    #[arg(long, default_value_t = 1)]
    start: u32,

    /// End author ID
    #[arg(long, default_value_t = 100)]
    end: u32,

    /// Sleep seconds between requests
    #[arg(long, default_value_t = 1)]
    sleep_sec: u64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.start > args.end {
        eprintln!("error: --start must be <= --end");
        std::process::exit(2);
    }

    let base_url = args.url.trim_end_matches('/');

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("wp_user_digger/0.3.0 (internal security audit)")
        .build()?;

    // Location ヘッダ用
    let header_author_re = Regex::new(r"/author/([^/]+)/?")?;
    // HTML 内 /author/<name>/ 用
    let html_author_re = Regex::new(r#"/author/([^/"'>\s]+)/?"#)?;
    // body class="author-bob"
    let body_author_re = Regex::new(r#"author-([a-zA-Z0-9_-]+)"#)?;

    let mut users: HashSet<String> = HashSet::new();

    for id in args.start..=args.end {
        let target = format!("{}/?author={}", base_url, id);

        if let Ok(resp) = client.get(&target).send() {
            /* ==========================
             * 1. Location ヘッダ解析
             * ========================== */
            if resp.status().is_redirection() {
                if let Some(loc) = resp.headers().get("Location") {
                    if let Ok(loc_str) = loc.to_str() {
                        if let Some(cap) = header_author_re.captures(loc_str) {
                            users.insert(cap[1].to_string());
                        }

                        // リダイレクト先 HTML も解析
                        let redirect_url = if loc_str.starts_with("http") {
                            loc_str.to_string()
                        } else {
                            format!("{}{}", base_url, loc_str)
                        };

                        if let Ok(html_resp) = client.get(&redirect_url).send() {
                            if let Ok(html) = html_resp.text() {
                                extract_from_html(&html, &html_author_re, &body_author_re, &mut users);
                            }
                        }
                    }
                }
            }
            /* ==========================
             * 2. リダイレクトしない場合
             * ========================== */
            else if let Ok(html) = resp.text() {
                extract_from_html(&html, &html_author_re, &body_author_re, &mut users);
            }
        }

        sleep(Duration::from_secs(args.sleep_sec));
    }

    for user in users {
        println!("{}", user);
    }

    Ok(())
}

fn extract_from_html(
    html: &str,
    author_url_re: &Regex,
    body_class_re: &Regex,
    users: &mut HashSet<String>,
) {
    /* ===============================
     * body class="author author-bob author-2"
     * =============================== */
    if let Some(cap) = body_class_re.captures(html) {
        let class_attr = &cap[1];
        let classes: Vec<&str> = class_attr.split_whitespace().collect();

        let mut name: Option<String> = None;
        let mut has_author_id = false;

        for class in &classes {
            if let Some(rest) = class.strip_prefix("author-") {
                if rest.chars().all(|c| c.is_ascii_digit()) {
                    has_author_id = true;
                } else if rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
                    name = Some(rest.to_string());
                }
            }
        }

        // author-<name> と author-<id> が揃っている場合のみ採用
        if has_author_id {
            if let Some(n) = name {
                users.insert(n);
            }
        }
    }

    /* ===============================
     * /author/<name>/ URL
     * =============================== */
    for cap in author_url_re.captures_iter(html) {
        users.insert(cap[1].to_string());
    }
}
