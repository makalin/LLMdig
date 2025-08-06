use crate::config::Config;
use crate::dns::DnsHandler;
use crate::Error;
use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tracing::{error, info, warn};
use trust_dns_proto::op::Message;
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};
use trust_dns_server::server::{Request, ResponseHandler, ResponseInfo};

pub struct DnsServer {
    config: Config,
    handler: Arc<DnsHandler>,
    socket: UdpSocket,
}

impl DnsServer {
    pub fn new(config: Config) -> Result<Self> {
        let handler = Arc::new(DnsHandler::new(config.clone())?);
        let addr = format!("{}:{}", config.server.host, config.server.port);
        let socket = UdpSocket::bind(&addr)?;
        
        info!("DNS server bound to {}", addr);

        Ok(Self {
            config,
            handler,
            socket,
        })
    }

    pub fn host(&self) -> &str {
        &self.config.server.host
    }

    pub fn port(&self) -> u16 {
        self.config.server.port
    }

    pub async fn run(&self) -> Result<()> {
        info!("Starting DNS server on {}:{}", self.host(), self.port());
        
        let mut buf = vec![0u8; 512];
        let handler = self.handler.clone();

        loop {
            match self.socket.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    let handler = handler.clone();
                    let data = buf[..len].to_vec();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_packet(handler, data, src).await {
                            error!("Error handling packet from {}: {}", src, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error receiving packet: {}", e);
                }
            }
        }
    }

    async fn handle_packet(
        handler: Arc<DnsHandler>,
        data: Vec<u8>,
        src: SocketAddr,
    ) -> Result<()> {
        // Parse DNS message
        let message = Message::from_bytes(&data)?;
        
        // Create request object
        let request = Request::new(message, src);
        
        // Create response handler
        let response_handler = Box::new(UdpResponseHandler::new(src));
        
        // Handle the request
        let _response_info = handler.handle_request(&request, response_handler).await?;
        
        Ok(())
    }
}

struct UdpResponseHandler {
    addr: SocketAddr,
}

impl UdpResponseHandler {
    fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

#[async_trait::async_trait]
impl ResponseHandler for UdpResponseHandler {
    async fn send_response(&self, response_bytes: Vec<u8>) -> Result<(), std::io::Error> {
        // For now, we'll just log the response
        // In a real implementation, you'd send it back via UDP
        info!("Would send {} bytes to {}", response_bytes.len(), self.addr);
        Ok(())
    }
} 