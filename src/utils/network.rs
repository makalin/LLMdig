use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub bind_address: IpAddr,
    pub port: u16,
    pub max_packet_size: usize,
    pub socket_buffer_size: usize,
    pub connection_timeout: Duration,
    pub read_timeout: Duration,
    pub write_timeout: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            bind_address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            port: 9000,
            max_packet_size: 512,
            socket_buffer_size: 65536,
            connection_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(10),
            write_timeout: Duration::from_secs(10),
        }
    }
}

#[derive(Debug)]
pub struct NetworkManager {
    config: NetworkConfig,
    socket: Option<UdpSocket>,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config,
            socket: None,
        }
    }

    pub async fn bind(&mut self) -> Result<(), std::io::Error> {
        let addr = SocketAddr::new(self.config.bind_address, self.config.port);
        
        info!("Binding to {}", addr);
        
        let socket = UdpSocket::bind(addr).await?;
        
        // Set socket options
        socket.set_recv_buffer_size(self.config.socket_buffer_size)?;
        socket.set_send_buffer_size(self.config.socket_buffer_size)?;
        
        // Set non-blocking mode
        socket.set_nonblocking(true)?;
        
        self.socket = Some(socket);
        
        info!("Successfully bound to {}", addr);
        Ok(())
    }

    pub async fn receive_packet(&self) -> Result<(Vec<u8>, SocketAddr), std::io::Error> {
        if let Some(socket) = &self.socket {
            let mut buffer = vec![0u8; self.config.max_packet_size];
            
            let (len, addr) = timeout(self.config.read_timeout, socket.recv_from(&mut buffer)).await
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "Receive timeout"))??;
            
            buffer.truncate(len);
            Ok((buffer, addr))
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Socket not bound"))
        }
    }

    pub async fn send_packet(&self, data: &[u8], addr: SocketAddr) -> Result<usize, std::io::Error> {
        if let Some(socket) = &self.socket {
            timeout(self.config.write_timeout, socket.send_to(data, addr)).await
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::TimedOut, "Send timeout"))??
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Socket not bound"))
        }
    }

    pub fn is_bound(&self) -> bool {
        self.socket.is_some()
    }

    pub fn get_local_addr(&self) -> Option<SocketAddr> {
        self.socket.as_ref()?.local_addr().ok()
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub packets_received: u64,
    pub packets_sent: u64,
    pub bytes_received: u64,
    pub bytes_sent: u64,
    pub errors: u64,
    pub timeouts: u64,
}

impl NetworkStats {
    pub fn new() -> Self {
        Self {
            packets_received: 0,
            packets_sent: 0,
            bytes_received: 0,
            bytes_sent: 0,
            errors: 0,
            timeouts: 0,
        }
    }
}

// DNS-specific network utilities
pub struct DnsNetworkUtils;

impl DnsNetworkUtils {
    /// Validate if a DNS packet is well-formed
    pub fn validate_dns_packet(data: &[u8]) -> bool {
        if data.len() < 12 {
            return false; // DNS header is 12 bytes
        }
        
        // Check DNS header flags
        let flags = u16::from_be_bytes([data[2], data[3]]);
        let qr = (flags >> 15) & 1; // Query/Response bit
        let opcode = (flags >> 11) & 0xF; // Opcode
        let rcode = flags & 0xF; // Response code
        
        // Basic validation
        if opcode > 5 {
            return false; // Invalid opcode
        }
        
        if qr == 1 && rcode > 5 {
            return false; // Invalid response code
        }
        
        true
    }

    /// Extract DNS query count from packet
    pub fn get_query_count(data: &[u8]) -> Option<u16> {
        if data.len() < 12 {
            return None;
        }
        
        Some(u16::from_be_bytes([data[4], data[5]]))
    }

    /// Extract DNS answer count from packet
    pub fn get_answer_count(data: &[u8]) -> Option<u16> {
        if data.len() < 12 {
            return None;
        }
        
        Some(u16::from_be_bytes([data[6], data[7]]))
    }

    /// Check if packet is a DNS query
    pub fn is_dns_query(data: &[u8]) -> bool {
        if data.len() < 12 {
            return false;
        }
        
        let flags = u16::from_be_bytes([data[2], data[3]]);
        let qr = (flags >> 15) & 1;
        
        qr == 0 // Query bit is 0
    }

    /// Check if packet is a DNS response
    pub fn is_dns_response(data: &[u8]) -> bool {
        if data.len() < 12 {
            return false;
        }
        
        let flags = u16::from_be_bytes([data[2], data[3]]);
        let qr = (flags >> 15) & 1;
        
        qr == 1 // Response bit is 1
    }

    /// Get DNS message ID
    pub fn get_dns_id(data: &[u8]) -> Option<u16> {
        if data.len() < 2 {
            return None;
        }
        
        Some(u16::from_be_bytes([data[0], data[1]]))
    }

    /// Set DNS message ID
    pub fn set_dns_id(data: &mut [u8], id: u16) -> bool {
        if data.len() < 2 {
            return false;
        }
        
        let id_bytes = id.to_be_bytes();
        data[0] = id_bytes[0];
        data[1] = id_bytes[1];
        true
    }
}

// Network diagnostics
pub struct NetworkDiagnostics;

impl NetworkDiagnostics {
    /// Test if a port is available for binding
    pub async fn test_port_availability(addr: SocketAddr) -> bool {
        match UdpSocket::bind(addr).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Find an available port in a range
    pub async fn find_available_port(start_port: u16, end_port: u16) -> Option<u16> {
        for port in start_port..=end_port {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
            if Self::test_port_availability(addr).await {
                return Some(port);
            }
        }
        None
    }

    /// Test DNS resolution
    pub async fn test_dns_resolution(domain: &str, nameserver: &str) -> Result<Duration, Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let start = std::time::Instant::now();
        
        let output = Command::new("dig")
            .arg("@".to_string() + nameserver)
            .arg(domain)
            .arg("+short")
            .arg("+timeout=5")
            .output()?;
        
        let duration = start.elapsed();
        
        if output.status.success() {
            Ok(duration)
        } else {
            Err("DNS resolution failed".into())
        }
    }

    /// Test network connectivity
    pub async fn test_connectivity(host: &str, port: u16) -> Result<Duration, Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", host, port);
        let start = std::time::Instant::now();
        
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(_) => Ok(start.elapsed()),
            Err(e) => Err(e.into()),
        }
    }

    /// Get network interface information
    pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            
            let mut interfaces = Vec::new();
            
            for entry in fs::read_dir("/sys/class/net")? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();
                
                if let Ok(addr) = fs::read_to_string(format!("/sys/class/net/{}/address", name)) {
                    interfaces.push(NetworkInterface {
                        name,
                        mac_address: addr.trim().to_string(),
                        ip_addresses: Vec::new(), // Would need more complex parsing
                    });
                }
            }
            
            Ok(interfaces)
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Fallback for other platforms
            Ok(vec![NetworkInterface {
                name: "default".to_string(),
                mac_address: "unknown".to_string(),
                ip_addresses: vec![],
            }])
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: String,
    pub ip_addresses: Vec<IpAddr>,
}

// Connection pool for managing multiple connections
pub struct ConnectionPool {
    max_connections: usize,
    connections: std::collections::HashMap<SocketAddr, UdpSocket>,
}

impl ConnectionPool {
    pub fn new(max_connections: usize) -> Self {
        Self {
            max_connections,
            connections: std::collections::HashMap::new(),
        }
    }

    pub async fn get_connection(&mut self, addr: SocketAddr) -> Result<&UdpSocket, std::io::Error> {
        if !self.connections.contains_key(&addr) {
            if self.connections.len() >= self.max_connections {
                // Remove oldest connection
                let oldest_addr = self.connections.keys().next().cloned();
                if let Some(old_addr) = oldest_addr {
                    self.connections.remove(&old_addr);
                }
            }
            
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            self.connections.insert(addr, socket);
        }
        
        Ok(self.connections.get(&addr).unwrap())
    }

    pub fn remove_connection(&mut self, addr: SocketAddr) {
        self.connections.remove(&addr);
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_packet_validation() {
        // Valid DNS query packet (minimal)
        let valid_query = vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(DnsNetworkUtils::validate_dns_packet(&valid_query));
        
        // Invalid packet (too short)
        let invalid_packet = vec![0x12, 0x34];
        assert!(!DnsNetworkUtils::validate_dns_packet(&invalid_packet));
    }

    #[test]
    fn test_dns_query_detection() {
        // DNS query
        let query = vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert!(DnsNetworkUtils::is_dns_query(&query));
        assert!(!DnsNetworkUtils::is_dns_response(&query));
        
        // DNS response
        let response = vec![0x12, 0x34, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00];
        assert!(!DnsNetworkUtils::is_dns_query(&response));
        assert!(DnsNetworkUtils::is_dns_response(&response));
    }

    #[test]
    fn test_dns_id_operations() {
        let mut packet = vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        
        assert_eq!(DnsNetworkUtils::get_dns_id(&packet), Some(0x1234));
        
        DnsNetworkUtils::set_dns_id(&mut packet, 0x5678);
        assert_eq!(DnsNetworkUtils::get_dns_id(&packet), Some(0x5678));
    }

    #[tokio::test]
    async fn test_network_manager() {
        let config = NetworkConfig {
            bind_address: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 0, // Use port 0 to let OS choose
            ..Default::default()
        };
        
        let mut manager = NetworkManager::new(config);
        assert!(manager.bind().await.is_ok());
        assert!(manager.is_bound());
    }
} 