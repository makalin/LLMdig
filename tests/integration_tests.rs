use llmdig::{Config, DnsHandler, LlmClient};
use std::net::SocketAddr;
use std::str::FromStr;
use trust_dns_proto::op::{Message, MessageType, OpCode, ResponseCode};
use trust_dns_proto::rr::{DNSClass, Name, RecordType};
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};
use trust_dns_server::server::{Request, ResponseHandler, ResponseInfo};

struct MockResponseHandler {
    responses: std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
}

impl MockResponseHandler {
    fn new() -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl ResponseHandler for MockResponseHandler {
    async fn send_response(&self, response_bytes: Vec<u8>) -> Result<(), std::io::Error> {
        self.responses.lock().unwrap().push(response_bytes);
        Ok(())
    }
}

#[tokio::test]
async fn test_dns_handler_basic_query() {
    let config = Config::default();
    let handler = DnsHandler::new(config).unwrap();
    
    // Create a mock DNS query
    let mut message = Message::new();
    message.set_id(1234);
    message.set_message_type(MessageType::Query);
    message.set_op_code(OpCode::Query);
    message.set_response_code(ResponseCode::NoError);
    
    let name = Name::from_str("what.is.the.weather.com").unwrap();
    let query = trust_dns_proto::op::Query::query(name, RecordType::TXT);
    message.add_query(query);
    
    let addr = SocketAddr::from_str("127.0.0.1:12345").unwrap();
    let request = Request::new(message, addr);
    
    let response_handler = Box::new(MockResponseHandler::new());
    
    // This will fail because we don't have a real LLM backend configured
    // but it should at least not panic
    let result = handler.handle_request(&request, response_handler).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_dns_handler_non_txt_query() {
    let config = Config::default();
    let handler = DnsHandler::new(config).unwrap();
    
    // Create a mock DNS query for A record (not TXT)
    let mut message = Message::new();
    message.set_id(1234);
    message.set_message_type(MessageType::Query);
    message.set_op_code(OpCode::Query);
    message.set_response_code(ResponseCode::NoError);
    
    let name = Name::from_str("example.com").unwrap();
    let query = trust_dns_proto::op::Query::query(name, RecordType::A);
    message.add_query(query);
    
    let addr = SocketAddr::from_str("127.0.0.1:12345").unwrap();
    let request = Request::new(message, addr);
    
    let response_handler = Box::new(MockResponseHandler::new());
    
    let result = handler.handle_request(&request, response_handler).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_llm_client_creation() {
    let config = Config::default();
    
    // This should fail because no API key is configured
    let result = LlmClient::new(config);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_loading() {
    // Test loading default config
    let config = Config::default();
    assert_eq!(config.server.port, 9000);
    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.llm.model, "gpt-3.5-turbo");
}

#[tokio::test]
async fn test_domain_parsing() {
    let test_cases = vec![
        ("what.is.the.weather.com", "what is the weather"),
        ("hello-world.example.com", "hello world example"),
        ("simple.test.com", "simple test"),
    ];
    
    for (domain, expected) in test_cases {
        let name = Name::from_str(domain).unwrap();
        let handler = DnsHandler::new(Config::default()).unwrap();
        let result = handler.extract_question_from_domain(&name);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected);
    }
}

#[tokio::test]
async fn test_invalid_domain_parsing() {
    let invalid_domains = vec![
        "single.com",
        "domain",
        "",
    ];
    
    let handler = DnsHandler::new(Config::default()).unwrap();
    
    for domain in invalid_domains {
        let name = Name::from_str(domain).unwrap();
        let result = handler.extract_question_from_domain(&name);
        assert!(result.is_err() || result.unwrap().is_empty());
    }
} 