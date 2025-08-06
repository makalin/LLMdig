#!/bin/bash

# LLMdig Benchmark Script
# This script performs various performance tests on the LLMdig server

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LLMDIG_HOST="localhost"
LLMDIG_PORT="9000"
BENCHMARK_DURATION=60
CONCURRENT_REQUESTS=10
TOTAL_REQUESTS=1000

# Test queries
TEST_QUERIES=(
    "what.is.the.weather.com"
    "how.many.stars.are.there.com"
    "what.is.the.capital.of.france.com"
    "hello.world.com"
    "test.query.com"
)

print_header() {
    echo -e "${BLUE}🔬 LLMdig Benchmark Suite${NC}"
    echo "================================"
    echo ""
}

print_section() {
    echo -e "${YELLOW}$1${NC}"
    echo "----------------------------------------"
}

print_result() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

check_prerequisites() {
    print_section "Checking Prerequisites"
    
    # Check if dig is available
    if ! command -v dig &> /dev/null; then
        print_error "dig command not found. Please install bind-utils or dnsutils."
        exit 1
    fi
    
    # Check if curl is available
    if ! command -v curl &> /dev/null; then
        print_error "curl command not found. Please install curl."
        exit 1
    fi
    
    # Check if LLMdig server is running
    if ! dig @$LLMDIG_HOST -p $LLMDIG_PORT "test.com" TXT +short &> /dev/null; then
        print_warning "LLMdig server doesn't seem to be running on $LLMDIG_HOST:$LLMDIG_PORT"
        print_warning "Start the server with: cargo run --release"
        echo ""
    fi
    
    print_result "All prerequisites satisfied"
    echo ""
}

test_basic_functionality() {
    print_section "Basic Functionality Test"
    
    local success_count=0
    local total_count=0
    
    for query in "${TEST_QUERIES[@]}"; do
        total_count=$((total_count + 1))
        
        if dig @$LLMDIG_HOST -p $LLMDIG_PORT "$query" TXT +short &> /dev/null; then
            success_count=$((success_count + 1))
            print_result "Query '$query' successful"
        else
            print_error "Query '$query' failed"
        fi
    done
    
    local success_rate=$((success_count * 100 / total_count))
    echo ""
    echo "Success Rate: $success_count/$total_count ($success_rate%)"
    echo ""
}

test_response_time() {
    print_section "Response Time Test"
    
    local total_time=0
    local count=0
    
    for query in "${TEST_QUERIES[@]}"; do
        local start_time=$(date +%s%N)
        
        if dig @$LLMDIG_HOST -p $LLMDIG_PORT "$query" TXT +short &> /dev/null; then
            local end_time=$(date +%s%N)
            local duration=$((end_time - start_time))
            local duration_ms=$((duration / 1000000))
            
            total_time=$((total_time + duration_ms))
            count=$((count + 1))
            
            print_result "Query '$query': ${duration_ms}ms"
        else
            print_error "Query '$query' failed"
        fi
    done
    
    if [ $count -gt 0 ]; then
        local avg_time=$((total_time / count))
        echo ""
        echo "Average Response Time: ${avg_time}ms"
        echo "Total Time: ${total_time}ms"
        echo "Successful Queries: $count"
    fi
    echo ""
}

test_concurrent_requests() {
    print_section "Concurrent Requests Test"
    
    local start_time=$(date +%s)
    local success_count=0
    local total_count=0
    
    # Run concurrent requests
    for i in $(seq 1 $TOTAL_REQUESTS); do
        local query="${TEST_QUERIES[$((i % ${#TEST_QUERIES[@]}))]}"
        
        (
            if dig @$LLMDIG_HOST -p $LLMDIG_PORT "$query" TXT +short &> /dev/null; then
                echo "success" >> /tmp/llmdig_benchmark_results
            else
                echo "failure" >> /tmp/llmdig_benchmark_results
            fi
        ) &
        
        # Limit concurrent processes
        if [ $((i % CONCURRENT_REQUESTS)) -eq 0 ]; then
            wait
        fi
    done
    
    # Wait for all processes to complete
    wait
    
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    # Count results
    if [ -f /tmp/llmdig_benchmark_results ]; then
        success_count=$(grep -c "success" /tmp/llmdig_benchmark_results || echo "0")
        total_count=$(wc -l < /tmp/llmdig_benchmark_results || echo "0")
        rm -f /tmp/llmdig_benchmark_results
    fi
    
    local success_rate=0
    if [ $total_count -gt 0 ]; then
        success_rate=$((success_count * 100 / total_count))
    fi
    
    local requests_per_second=0
    if [ $duration -gt 0 ]; then
        requests_per_second=$((total_count / duration))
    fi
    
    echo "Duration: ${duration}s"
    echo "Total Requests: $total_count"
    echo "Successful Requests: $success_count"
    echo "Success Rate: $success_rate%"
    echo "Requests per Second: $requests_per_second"
    echo ""
}

test_memory_usage() {
    print_section "Memory Usage Test"
    
    if ! pgrep -f "llmdig" > /dev/null; then
        print_warning "LLMdig process not found, skipping memory test"
        echo ""
        return
    fi
    
    local pid=$(pgrep -f "llmdig" | head -1)
    if [ -n "$pid" ]; then
        local memory_kb=$(ps -o rss= -p "$pid" 2>/dev/null || echo "0")
        local memory_mb=$((memory_kb / 1024))
        
        echo "LLMdig Process ID: $pid"
        echo "Memory Usage: ${memory_kb}KB (${memory_mb}MB)"
        
        # Get additional process info
        if command -v pmap &> /dev/null; then
            echo ""
            echo "Memory Map (top 10 regions):"
            pmap -x "$pid" 2>/dev/null | head -12 || true
        fi
    else
        print_warning "Could not find LLMdig process"
    fi
    echo ""
}

test_network_connectivity() {
    print_section "Network Connectivity Test"
    
    # Test basic connectivity
    if nc -z $LLMDIG_HOST $LLMDIG_PORT 2>/dev/null; then
        print_result "Port $LLMDIG_PORT is open on $LLMDIG_HOST"
    else
        print_error "Port $LLMDIG_PORT is not accessible on $LLMDIG_HOST"
    fi
    
    # Test DNS resolution
    if dig @$LLMDIG_HOST -p $LLMDIG_PORT "health.check" TXT +short &> /dev/null; then
        print_result "DNS server is responding"
    else
        print_error "DNS server is not responding"
    fi
    
    # Test with different query types
    local query_types=("TXT" "A" "AAAA" "MX")
    for qtype in "${query_types[@]}"; do
        if dig @$LLMDIG_HOST -p $LLMDIG_PORT "test.com" "$qtype" +short &> /dev/null; then
            print_result "Query type $qtype is supported"
        else
            print_warning "Query type $qtype may not be supported"
        fi
    done
    echo ""
}

test_load_simulation() {
    print_section "Load Simulation Test"
    
    local duration=$BENCHMARK_DURATION
    local start_time=$(date +%s)
    local end_time=$((start_time + duration))
    local current_time=$start_time
    local request_count=0
    local success_count=0
    
    echo "Running load simulation for ${duration} seconds..."
    echo "Target: $CONCURRENT_REQUESTS concurrent requests"
    echo ""
    
    while [ $current_time -lt $end_time ]; do
        # Start concurrent requests
        for i in $(seq 1 $CONCURRENT_REQUESTS); do
            local query="${TEST_QUERIES[$((RANDOM % ${#TEST_QUERIES[@]}))]}"
            
            (
                local req_start=$(date +%s%N)
                if dig @$LLMDIG_HOST -p $LLMDIG_PORT "$query" TXT +short &> /dev/null; then
                    local req_end=$(date +%s%N)
                    local req_duration=$((req_end - req_start))
                    echo "success $req_duration" >> /tmp/llmdig_load_test
                else
                    echo "failure 0" >> /tmp/llmdig_load_test
                fi
            ) &
        done
        
        # Wait for current batch
        wait
        
        current_time=$(date +%s)
        request_count=$((request_count + CONCURRENT_REQUESTS))
        
        # Progress indicator
        local elapsed=$((current_time - start_time))
        local progress=$((elapsed * 100 / duration))
        echo -ne "\rProgress: ${progress}% (${elapsed}s/${duration}s)"
    done
    
    echo ""
    echo ""
    
    # Process results
    if [ -f /tmp/llmdig_load_test ]; then
        success_count=$(grep -c "success" /tmp/llmdig_load_test || echo "0")
        total_requests=$(wc -l < /tmp/llmdig_load_test || echo "0")
        
        # Calculate average response time
        local total_time=0
        local time_count=0
        while IFS=' ' read -r result duration; do
            if [ "$result" = "success" ] && [ "$duration" -gt 0 ]; then
                total_time=$((total_time + duration))
                time_count=$((time_count + 1))
            fi
        done < /tmp/llmdig_load_test
        
        local avg_response_time=0
        if [ $time_count -gt 0 ]; then
            avg_response_time=$((total_time / time_count / 1000000)) # Convert to ms
        fi
        
        local success_rate=0
        if [ $total_requests -gt 0 ]; then
            success_rate=$((success_count * 100 / total_requests))
        fi
        
        local rps=0
        if [ $duration -gt 0 ]; then
            rps=$((total_requests / duration))
        fi
        
        echo "Load Test Results:"
        echo "  Duration: ${duration}s"
        echo "  Total Requests: $total_requests"
        echo "  Successful Requests: $success_count"
        echo "  Success Rate: $success_rate%"
        echo "  Requests per Second: $rps"
        echo "  Average Response Time: ${avg_response_time}ms"
        
        rm -f /tmp/llmdig_load_test
    fi
    echo ""
}

generate_report() {
    print_section "Benchmark Report"
    
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local report_file="llmdig_benchmark_$(date +%Y%m%d_%H%M%S).txt"
    
    {
        echo "LLMdig Benchmark Report"
        echo "Generated: $timestamp"
        echo "Server: $LLMDIG_HOST:$LLMDIG_PORT"
        echo "========================================"
        echo ""
        echo "Test Configuration:"
        echo "  Benchmark Duration: ${BENCHMARK_DURATION}s"
        echo "  Concurrent Requests: $CONCURRENT_REQUESTS"
        echo "  Total Requests: $TOTAL_REQUESTS"
        echo ""
        echo "System Information:"
        echo "  OS: $(uname -s)"
        echo "  Architecture: $(uname -m)"
        echo "  CPU: $(nproc) cores"
        echo "  Memory: $(free -h | grep Mem | awk '{print $2}')"
        echo ""
    } > "$report_file"
    
    print_result "Report saved to: $report_file"
    echo ""
}

# Main execution
main() {
    print_header
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --host)
                LLMDIG_HOST="$2"
                shift 2
                ;;
            --port)
                LLMDIG_PORT="$2"
                shift 2
                ;;
            --duration)
                BENCHMARK_DURATION="$2"
                shift 2
                ;;
            --concurrent)
                CONCURRENT_REQUESTS="$2"
                shift 2
                ;;
            --requests)
                TOTAL_REQUESTS="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --host HOST        LLMdig server host (default: localhost)"
                echo "  --port PORT        LLMdig server port (default: 9000)"
                echo "  --duration SECONDS Benchmark duration (default: 60)"
                echo "  --concurrent N     Concurrent requests (default: 10)"
                echo "  --requests N       Total requests (default: 1000)"
                echo "  --help             Show this help message"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    check_prerequisites
    test_basic_functionality
    test_response_time
    test_concurrent_requests
    test_memory_usage
    test_network_connectivity
    test_load_simulation
    generate_report
    
    echo -e "${GREEN}🎉 Benchmark completed successfully!${NC}"
}

# Run main function
main "$@" 