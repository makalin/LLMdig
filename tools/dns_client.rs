use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::net::UdpSocket;
use trust_dns_proto::op::{Message, MessageType, OpCode, ResponseCode};
use trust_dns_proto::rr::{DNSClass, Name, RecordType};
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    
    /// DNS server host
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,
    
    /// DNS server port
    #[arg(short, long, default_value = "9000")]
    port: u16,
    
    /// Timeout in seconds
    #[arg(short, long, default_value = "10")]
    timeout: u64,
}

#[derive(Subcommand)]
enum Commands {
    /// Query a domain
    Query {
        /// Domain to query
        domain: String,
        
        /// Record type
        #[arg(short, long, default_value = "TXT")]
        record_type: String,
    },
    
    /// Batch query multiple domains
    Batch {
        /// File containing domains (one per line)
        file: String,
        
        /// Record type
        #[arg(short, long, default_value = "TXT")]
        record_type: String,
        
        /// Concurrent requests
        #[arg(short, long, default_value = "10")]
        concurrent: usize,
    },
    
    /// Health check
    Health,
    
    /// Performance test
    Perf {
        /// Number of requests
        #[arg(short, long, default_value = "100")]
        requests: usize,
        
        /// Concurrent requests
        #[arg(short, long, default_value = "10")]
        concurrent: usize,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    let server_addr = format!("{}:{}", args.host, args.port);
    let socket_addr = SocketAddr::from_str(&server_addr)?;
    
    match args.command {
        Commands::Query { domain, record_type } => {
            query_domain(&socket_addr, &domain, &record_type, args.timeout).await?;
        }
        Commands::Batch { file, record_type, concurrent } => {
            batch_query(&socket_addr, &file, &record_type, concurrent, args.timeout).await?;
        }
        Commands::Health => {
            health_check(&socket_addr, args.timeout).await?;
        }
        Commands::Perf { requests, concurrent } => {
            performance_test(&socket_addr, requests, concurrent, args.timeout).await?;
        }
    }
    
    Ok(())
}

async fn query_domain(
    server_addr: &SocketAddr,
    domain: &str,
    record_type: &str,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Querying {} {} from {}", domain, record_type, server_addr);
    
    let start_time = std::time::Instant::now();
    let response = send_dns_query(server_addr, domain, record_type, timeout).await?;
    let duration = start_time.elapsed();
    
    println!("Response time: {:?}", duration);
    println!("Response: {:?}", response);
    
    Ok(())
}

async fn batch_query(
    server_addr: &SocketAddr,
    file: &str,
    record_type: &str,
    concurrent: usize,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let domains = std::fs::read_to_string(file)?
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    
    println!("Batch querying {} domains with {} concurrent requests", domains.len(), concurrent);
    
    let start_time = std::time::Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;
    
    // Process domains in batches
    for chunk in domains.chunks(concurrent) {
        let mut handles = vec![];
        
        for domain in chunk {
            let server_addr = *server_addr;
            let domain = domain.clone();
            let record_type = record_type.to_string();
            
            handles.push(tokio::spawn(async move {
                send_dns_query(&server_addr, &domain, &record_type, timeout).await
            }));
        }
        
        for handle in handles {
            match handle.await? {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
    }
    
    let duration = start_time.elapsed();
    
    println!("Batch query completed in {:?}", duration);
    println!("Success: {}, Errors: {}", success_count, error_count);
    println!("Average time per query: {:?}", duration / domains.len() as u32);
    
    Ok(())
}

async fn health_check(
    server_addr: &SocketAddr,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing health check on {}", server_addr);
    
    let start_time = std::time::Instant::now();
    let result = send_dns_query(server_addr, "health.check", "TXT", timeout).await;
    let duration = start_time.elapsed();
    
    match result {
        Ok(response) => {
            println!("✓ Health check passed");
            println!("Response time: {:?}", duration);
            println!("Response: {:?}", response);
        }
        Err(e) => {
            println!("✗ Health check failed: {}", e);
        }
    }
    
    Ok(())
}

async fn performance_test(
    server_addr: &SocketAddr,
    requests: usize,
    concurrent: usize,
    timeout: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performance test: {} requests, {} concurrent", requests, concurrent);
    
    let test_domains = vec![
        "what.is.the.weather.com",
        "how.many.stars.are.there.com",
        "what.is.the.capital.of.france.com",
        "hello.world.com",
        "test.query.com",
    ];
    
    let start_time = std::time::Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;
    let mut response_times = Vec::new();
    
    // Process requests in batches
    for chunk_start in (0..requests).step_by(concurrent) {
        let chunk_end = std::cmp::min(chunk_start + concurrent, requests);
        let mut handles = vec![];
        
        for i in chunk_start..chunk_end {
            let server_addr = *server_addr;
            let domain = test_domains[i % test_domains.len()].to_string();
            
            handles.push(tokio::spawn(async move {
                let req_start = std::time::Instant::now();
                let result = send_dns_query(&server_addr, &domain, "TXT", timeout).await;
                let req_duration = req_start.elapsed();
                (result, req_duration)
            }));
        }
        
        for handle in handles {
            match handle.await? {
                (Ok(_), duration) => {
                    success_count += 1;
                    response_times.push(duration);
                }
                (Err(_), _) => error_count += 1,
            }
        }
    }
    
    let total_duration = start_time.elapsed();
    
    // Calculate statistics
    let avg_response_time = if !response_times.is_empty() {
        response_times.iter().sum::<std::time::Duration>() / response_times.len() as u32
    } else {
        std::time::Duration::ZERO
    };
    
    let min_response_time = response_times.iter().min().unwrap_or(&std::time::Duration::ZERO);
    let max_response_time = response_times.iter().max().unwrap_or(&std::time::Duration::ZERO);
    
    let requests_per_second = if total_duration.as_secs() > 0 {
        requests as f64 / total_duration.as_secs_f64()
    } else {
        0.0
    };
    
    println!("Performance test completed");
    println!("Total time: {:?}", total_duration);
    println!("Total requests: {}", requests);
    println!("Successful requests: {}", success_count);
    println!("Failed requests: {}", error_count);
    println!("Success rate: {:.2}%", (success_count as f64 / requests as f64) * 100.0);
    println!("Requests per second: {:.2}", requests_per_second);
    println!("Average response time: {:?}", avg_response_time);
    println!("Min response time: {:?}", min_response_time);
    println!("Max response time: {:?}", max_response_time);
    
    Ok(())
}

async fn send_dns_query(
    server_addr: &SocketAddr,
    domain: &str,
    record_type: &str,
    timeout: u64,
) -> Result<Message, Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server_addr).await?;
    
    // Create DNS query
    let mut message = Message::new();
    message.set_id(rand::random());
    message.set_message_type(MessageType::Query);
    message.set_op_code(OpCode::Query);
    message.set_response_code(ResponseCode::NoError);
    
    let name = Name::from_str(domain)?;
    let record_type = RecordType::from_str(record_type)?;
    let query = trust_dns_proto::op::Query::query(name, record_type);
    message.add_query(query);
    
    // Send query
    let query_bytes = message.to_bytes()?;
    socket.send(&query_bytes).await?;
    
    // Receive response
    let mut response_buffer = vec![0u8; 512];
    let len = tokio::time::timeout(
        std::time::Duration::from_secs(timeout),
        socket.recv(&mut response_buffer)
    ).await??;
    
    response_buffer.truncate(len);
    
    // Parse response
    let response = Message::from_bytes(&response_buffer)?;
    Ok(response)
} 