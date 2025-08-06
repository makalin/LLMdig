use crate::config::Config;
use crate::llm::LlmClient;
use crate::utils::rate_limiter::RateLimiter;
use crate::Error;
use anyhow::Result;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use trust_dns_proto::op::{Message, MessageType, ResponseCode};
use trust_dns_proto::rr::{DNSClass, Name, Record, RecordType};
use trust_dns_proto::serialize::binary::{BinDecodable, BinEncodable};
use trust_dns_server::authority::{Authority, Catalog};
use trust_dns_server::server::{Request, ResponseHandler, ResponseInfo};

pub struct DnsHandler {
    llm_client: LlmClient,
    config: Config,
    rate_limiter: Arc<RateLimiter>,
    cache: Arc<RwLock<HashMap<String, (String, std::time::Instant)>>>,
}

impl DnsHandler {
    pub fn new(config: Config) -> Result<Self> {
        let llm_client = LlmClient::new(config.clone())?;
        let rate_limiter = Arc::new(RateLimiter::new(
            config.rate_limit.requests_per_minute,
            config.rate_limit.burst_size,
        ));

        Ok(Self {
            llm_client,
            config,
            rate_limiter,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn handle_request(
        &self,
        request: &Request,
        response_handle: Box<dyn ResponseHandler>,
    ) -> Result<ResponseInfo> {
        let client_addr = request.src();
        let query = request.query();

        info!(
            "DNS query from {}: {:?} {:?}",
            client_addr, query.name(), query.query_type()
        );

        // Check rate limiting
        if self.config.rate_limit.enabled {
            if !self.rate_limiter.allow_request(client_addr).await {
                warn!("Rate limit exceeded for {}", client_addr);
                return self.send_error_response(request, ResponseCode::ServFail, response_handle).await;
            }
        }

        // Only handle TXT queries
        if query.query_type() != RecordType::TXT {
            debug!("Ignoring non-TXT query: {:?}", query.query_type());
            return self.send_error_response(request, ResponseCode::NotImp, response_handle).await;
        }

        // Extract question from domain name
        let question = self.extract_question_from_domain(query.name())?;
        
        if question.is_empty() {
            warn!("Empty question extracted from domain");
            return self.send_error_response(request, ResponseCode::FormErr, response_handle).await;
        }

        // Check cache first
        if let Some((cached_response, timestamp)) = self.cache.read().await.get(&question) {
            if timestamp.elapsed().as_secs() < 300 { // 5 minute cache
                info!("Returning cached response for: {}", question);
                return self.send_txt_response(request, cached_response, response_handle).await;
            }
        }

        // Generate LLM response
        match self.llm_client.query(&question).await {
            Ok(response) => {
                // Cache the response
                self.cache.write().await.insert(
                    question.clone(),
                    (response.clone(), std::time::Instant::now()),
                );

                info!("Generated response for: {}", question);
                self.send_txt_response(request, &response, response_handle).await
            }
            Err(e) => {
                error!("LLM query failed: {}", e);
                self.send_error_response(request, ResponseCode::ServFail, response_handle).await
            }
        }
    }

    fn extract_question_from_domain(&self, domain: &Name) -> Result<String> {
        let domain_str = domain.to_string();
        
        // Remove trailing dot if present
        let domain_str = domain_str.trim_end_matches('.');
        
        // Split by dots and reverse to get the question
        let parts: Vec<&str> = domain_str.split('.').collect();
        
        if parts.len() < 2 {
            return Err(Error::InvalidQuery("Domain must have at least 2 parts".to_string()).into());
        }

        // The question is everything except the last part (which is the TLD)
        let question_parts = &parts[..parts.len() - 1];
        let question = question_parts.join(" ");
        
        // Clean up the question
        let question = question.replace('-', " ").replace('_', " ");
        
        Ok(question)
    }

    async fn send_txt_response(
        &self,
        request: &Request,
        response_text: &str,
        response_handle: Box<dyn ResponseHandler>,
    ) -> Result<ResponseInfo> {
        let query = request.query();
        let mut response = Message::new();
        
        response.set_id(request.id());
        response.set_message_type(MessageType::Response);
        response.set_op_code(request.op_code());
        response.set_response_code(ResponseCode::NoError);
        response.set_authoritative(true);
        response.set_recursion_desired(request.recursion_desired());
        response.set_recursion_available(false);
        response.set_authentic_data(false);
        response.set_checking_disabled(false);
        response.set_query(query.clone());

        // Split response into chunks that fit in TXT records (255 bytes max per string)
        let chunks = self.chunk_response(response_text);
        
        for chunk in chunks {
            let record = Record::from_rdata(
                query.name().clone(),
                300, // TTL
                trust_dns_proto::rr::RData::TXT(chunk),
            );
            response.add_answer(record);
        }

        let response_bytes = response.to_bytes()?;
        response_handle.send_response(response_bytes).await?;
        
        Ok(ResponseInfo::new(
            request.id(),
            ResponseCode::NoError,
            false,
        ))
    }

    async fn send_error_response(
        &self,
        request: &Request,
        response_code: ResponseCode,
        response_handle: Box<dyn ResponseHandler>,
    ) -> Result<ResponseInfo> {
        let query = request.query();
        let mut response = Message::new();
        
        response.set_id(request.id());
        response.set_message_type(MessageType::Response);
        response.set_op_code(request.op_code());
        response.set_response_code(response_code);
        response.set_authoritative(true);
        response.set_recursion_desired(request.recursion_desired());
        response.set_recursion_available(false);
        response.set_authentic_data(false);
        response.set_checking_disabled(false);
        response.set_query(query.clone());

        let response_bytes = response.to_bytes()?;
        response_handle.send_response(response_bytes).await?;
        
        Ok(ResponseInfo::new(request.id(), response_code, false))
    }

    fn chunk_response(&self, response: &str) -> Vec<Vec<u8>> {
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();
        
        for byte in response.bytes() {
            if current_chunk.len() >= 255 {
                chunks.push(current_chunk);
                current_chunk = Vec::new();
            }
            current_chunk.push(byte);
        }
        
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }
        
        if chunks.is_empty() {
            chunks.push(b"No response".to_vec());
        }
        
        chunks
    }
} 