mod processor;
use clap::Parser;
use serde_json::json;
use std::process;
use std::time::Instant;

#[derive(Parser)]
#[command(name = "OmniPull URL Processor")]
#[command(about = "Blazingly fast URL processor for OmniPull Download Manager", long_about = None)]
struct Cli {
    /// The URL to process
    url: String,

    /// Timeout in seconds
    #[arg(short, long, default_value_t = 30)]
    timeout: u64,

    /// Output format (json or pretty)
    #[arg(short, long, default_value = "json")]
    format: String,

    /// Custom User Agent
    #[arg(short, long)]
    user_agent: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let start_time = Instant::now();

    match processor::process(&cli.url, cli.timeout, None) {
        Ok(info) => {
            let processing_time_ms = start_time.elapsed().as_millis() as u64;

            if cli.format == "pretty" {
                println!("URL Information:");
                println!("  URL:           {}", cli.url);
                println!("  Final URL:     {}", info.real_url.as_deref().unwrap_or("N/A"));
                println!("  Filename:      {}", info.filename.unwrap_or_else(|| "Unknown".into()));
                println!("  Size:          {} bytes", info.size.unwrap_or(0));
                println!("  Content-Type:  {}", info.content_type.unwrap_or_else(|| "Unknown".into()));
                println!("  Is Direct:     {}", info.is_direct);
                println!("  Is Supported:  {}", info.is_supported);
                println!("  Status Code:   {}", info.status_code);
                println!("  Processing Time: {} ms", processing_time_ms);
            } else {
                let output = json!({
                    "url": cli.url,
                    "final_url": info.real_url,
                    "filename": info.filename,
                    "size": info.size,
                    "content_type": info.content_type,
                    "is_direct": info.is_direct,
                    "is_supported": info.is_supported,
                    "status_code": info.status_code,
                    "processing_time_ms": processing_time_ms,
                    "error": info.error_msg
                });
                println!("{}", output);
            }

            if info.is_supported {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error occurred: {}", e);
            process::exit(2);
        }
    }
}