



/////////////////////////////////// WORKING NOW ///////////////////////////////////////////////////////////////////

use anyhow::{Context, Result};
use std::time::Duration;
use url::Url;
use serde_json::Value;
use log::{debug, warn};

// #[derive(Debug, Clone, Default)] // Add Default here
pub struct UrlInfo {
    pub is_supported: bool,
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub content_type: Option<String>,
    pub real_url: Option<String>,
    pub error_msg: Option<String>,
    pub status_code: u16,
    pub is_direct: bool,
}

const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub fn process(url: &str, timeout: u64, referer: Option<String>) -> Result<UrlInfo> {
    let parsed = Url::parse(url).context("Invalid URL")?;
    let host = parsed.host_str().unwrap_or("").to_lowercase();

    // 1. TikTok Logic
    if host.contains("tiktok.com") || host.contains("douyin.com") {
        return process_tiktok(url, timeout);
    }

    // 2. Instagram & CDN Logic
    // 2. Instagram & CDN Logic - Fixes the 0-byte issue
    if host.contains("instagram.com") || host.contains("fbcdn.net") {
        return process_instagram(url, timeout, referer);
    }

    if host.contains("kwik.cx") {
        if url.contains("vault") {
            return process_kwik_vault(url, timeout, referer);
        } else {
            // If it's the landing page link, handle it with the POST logic
            return process_kwik(url, timeout, referer); 
        }
    }

    // Default direct file logic
    process_direct(url, timeout, referer, &parsed)
}


fn process_tiktok(url: &str, timeout: u64) -> Result<UrlInfo> {
    // Ensure the URL is clean and starts with the proper domain for the API
    let clean_url = url.split('?').next().unwrap_or(url);
    // Try a direct ID-based API call if the URL format is weird
    let api_url = format!("https://www.tikwm.com/api/?url={}", clean_url);
    // ... API call logic
    
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(timeout))
        .build();

    let resp = agent.get(&api_url)
        .set("User-Agent", USER_AGENT)
        .call()?;

    let status_code = resp.status();
    
    let json: Value = serde_json::from_reader(resp.into_reader())
        .context("Failed to parse TikTok API JSON")?;

    if json["code"].as_i64().unwrap_or(-1) == 0 {
        let data = &json["data"];
        Ok(UrlInfo {
            is_supported: true,
            filename: Some(format!("{}.mp4", data["title"].as_str().unwrap_or("tiktok_video"))),
            size: data["size"].as_u64().or(data["wm_size"].as_u64()),
            content_type: Some("video/mp4".into()),
            real_url: data["play"].as_str().map(|s: &str| s.to_string()),
            error_msg: None,
            status_code,
            is_direct: true,
        })
    } else {
        Ok(UrlInfo {
            is_supported: false,
            filename: None,
            size: None,
            content_type: None,
            real_url: None,
            error_msg: Some(json["msg"].as_str().unwrap_or("TikTok API Error").to_string()),
            status_code,
            is_direct: false,
        })
    }
}



fn process_instagram(url: &str, timeout: u64, referer: Option<String>) -> Result<UrlInfo> {
    let is_cdn = url.contains("fbcdn.net");
    
    let extracted_filename = if is_cdn {
        url.split('?').next()
           .and_then(|path| path.split('/').last())
           .unwrap_or("instagram_image.jpg")
           .to_string()
    } else {
        url.split("/").filter(|s| !s.is_empty()).last()
           .map(|id| format!("IG_{}.mp4", id))
           .unwrap_or_else(|| "instagram_video.mp4".to_string())
    };

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout))
        .build();

    let mut size: Option<u64> = None;
    let mut content_type: Option<String> = None;
    let mut status_code: u16 = 0;
    
    if is_cdn {
        // CRITICAL FIX: Use the referrer passed from the background script
        // Default to Instagram referrer if none provided
        let final_referer = referer.as_deref().unwrap_or("https://www.instagram.com/");
        
        if let Ok(resp) = agent.head(url)
            .set("Referer", final_referer)
            .set("User-Agent", USER_AGENT)
            .call() {
            status_code = resp.status();
            size = resp.header("Content-Length").and_then(|s| s.parse::<u64>().ok());
            content_type = resp.header("Content-Type").map(|s| s.to_string());
            
            debug!("[Instagram] HEAD request successful: size={:?}, type={:?}", size, content_type);
        } else {
            warn!("[Instagram] HEAD request failed for CDN URL");
        }
    }

    Ok(UrlInfo {
        is_supported: true,
        filename: Some(extracted_filename),
        size,
        content_type: content_type.or(Some(if is_cdn { "image/jpeg".into() } else { "video/mp4".into() })),
        real_url: Some(url.to_string()),
        error_msg: None,
        status_code,
        is_direct: true,
    })
}





fn process_kwik(url: &str, timeout: u64, referer: Option<String>) -> Result<UrlInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout))
        .build();

    // IMPORTANT: Kwik Vault links will ALWAYS return 2KB if the referer is not kwik.cx
    let ref_str = referer.unwrap_or_else(|| "https://kwik.cx/".to_string());

    let resp = agent.get(url)
        .set("Referer", &ref_str)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .call()
        .context("Failed to connect to Kwik Vault")?;

    let status_code = resp.status();

    let parsed = Url::parse(url)?;
    
    // Get the filename from the ?file= parameter
    let filename = parsed.query_pairs()
        .find(|(key, _)| key == "file")
        .map(|(_, val)| val.to_string())
        .unwrap_or_else(|| "Anime_Video.mp4".into());

    Ok(UrlInfo {
        is_supported: true,
        filename: Some(filename),
        size: resp.header("Content-Length").and_then(|s| s.parse().ok()),
        content_type: Some("video/mp4".into()),
        real_url: Some(url.to_string()),
        error_msg: None,
        status_code,
        is_direct: true,
    })
}


fn process_kwik_vault(url: &str, timeout: u64, referer: Option<String>) -> Result<UrlInfo> {
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(timeout))
        .build();

    // Vault links REQUIRE the kwik.cx referer or they return 2KB
    let ref_str = referer.unwrap_or_else(|| "https://kwik.cx/".to_string());

    let resp = agent.head(url)
        .set("Referer", &ref_str)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .call()
        .context("Vault connection failed")?;

    let status_code = resp.status();

    let parsed = Url::parse(url)?;
    
    // Extract the pretty filename from '?file=...'
    let filename = parsed.query_pairs()
        .find(|(key, _)| key == "file")
        .map(|(_, val)| val.to_string())
        .unwrap_or_else(|| "anime_video.mp4".into());

    Ok(UrlInfo {
        is_supported: true,
        filename: Some(filename),
        size: resp.header("Content-Length").and_then(|s| s.parse().ok()),
        content_type: Some("video/mp4".into()),
        real_url: Some(url.to_string()),
        error_msg: None,
        status_code,
        is_direct: true,
    })
}



fn process_direct(url: &str, timeout: u64, referer: Option<String>, parsed: &Url) -> Result<UrlInfo> {
    let fallback_referer = format!("https://{}/", parsed.host_str().unwrap_or(""));
    let final_referer = referer.as_deref().unwrap_or(&fallback_referer);

    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(timeout))
        .redirects(10)
        .build();

    let request = agent.get(url)
        .set("User-Agent", USER_AGENT)
        .set("Referer", final_referer)
        .set("Range", "bytes=0-0")
        .set("Accept", "*/*")
        .set("Accept-Language", "en-US,en;q=0.9")
        .set("Sec-Fetch-Dest", "video")
        .set("Sec-Fetch-Mode", "no-cors")
        .set("Sec-Fetch-Site", "cross-site");

    let resp = match request.call() {
        Ok(r) => r,
        Err(e) => return Ok(UrlInfo { 
            is_supported: false, filename: None, size: None, content_type: None,
            real_url: None,
            error_msg: Some(format!("HTTP Error: {}", e)),
            status_code: 0,
            is_direct: false,
        }),
    };

    let status_code = resp.status();
    let content_type = resp.header("content-type").map(|s| s.to_string());
    
    let size = if let Some(cr) = resp.header("content-range") {
        cr.split('/').last().and_then(|s| s.parse::<u64>().ok())
    } else {
        resp.header("content-length").and_then(|s| s.parse::<u64>().ok())
    };

    let filename = if let Some(disp) = resp.header("content-disposition") {
        parse_content_disposition(disp)
    } else {
        parsed.path_segments()
            .and_then(|s| s.last())
            .map(|s| urlencoding::decode(s).unwrap_or(s.into()).to_string())
    };

    let (is_direct, reason) = is_direct_file(content_type.as_deref(), size);

    Ok(UrlInfo {
        is_supported: is_direct,
        filename,
        size,
        content_type,
        real_url: Some(url.to_string()),
        error_msg: reason,
        status_code,
        is_direct,
    })
}

fn parse_content_disposition(header: &str) -> Option<String> {
    if let Some(utf_part) = header.split(';').find(|p| p.trim().starts_with("filename*=")) {
        if let Some(val) = utf_part.split("''").last() {
            return urlencoding::decode(val).ok().map(|s| s.into_owned());
        }
    }
    header.split(';').find_map(|part| {
        let p = part.trim();
        if p.to_lowercase().starts_with("filename=") {
            Some(p.trim_start_matches("filename=").trim_matches('\"').to_string())
        } else { None }
    })
}

fn is_direct_file(ct: Option<&str>, size: Option<u64>) -> (bool, Option<String>) {
    if let Some(c) = ct {
        let low = c.to_lowercase();
        if low.contains("text/html") || low.contains("text/plain") {
            return (false, Some("Rejected: Server returned text/html error page".to_string()));
        }
    }
    let size_val = size.unwrap_or(0);
    if size_val < 5120 {
        if ct.is_none() { return (false, Some("Rejected: Tiny file/No type".into())); }
        if let Some(c) = ct {
            if c.contains("video/") || c.contains("application/zip") {
                return (false, Some(format!("Rejected: Fake File ({} bytes)", size_val)));
            }
        }
    }
    (true, None)
}