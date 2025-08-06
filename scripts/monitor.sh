#!/bin/bash

# LLMdig Monitoring Script
# Real-time monitoring of LLMdig server performance and health

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
LLMDIG_HOST="localhost"
LLMDIG_PORT="9000"
MONITOR_INTERVAL=5
LOG_FILE="/tmp/llmdig_monitor.log"
METRICS_FILE="/tmp/llmdig_metrics.json"

# ANSI escape codes for terminal manipulation
CLEAR_LINE="\033[K"
MOVE_UP="\033[A"
MOVE_DOWN="\033[B"

print_header() {
    echo -e "${BLUE}📊 LLMdig Server Monitor${NC}"
    echo "================================"
    echo "Server: $LLMDIG_HOST:$LLMDIG_PORT"
    echo "Interval: ${MONITOR_INTERVAL}s"
    echo "Press Ctrl+C to stop monitoring"
    echo ""
}

print_section() {
    echo -e "${YELLOW}$1${NC}"
}

print_metric() {
    local label="$1"
    local value="$2"
    local color="$3"
    echo -e "${color}${label}:${NC} $value"
}

print_status() {
    local status="$1"
    local message="$2"
    if [ "$status" = "OK" ]; then
        echo -e "${GREEN}✓${NC} $message"
    elif [ "$status" = "WARNING" ]; then
        echo -e "${YELLOW}⚠${NC} $message"
    else
        echo -e "${RED}✗${NC} $message"
    fi
}

get_timestamp() {
    date '+%Y-%m-%d %H:%M:%S'
}

check_server_status() {
    local start_time=$(date +%s%N)
    
    if dig @$LLMDIG_HOST -p $LLMDIG_PORT "health.check" TXT +short &> /dev/null; then
        local end_time=$(date +%s%N)
        local duration=$((end_time - start_time))
        local duration_ms=$((duration / 1000000))
        echo "OK:$duration_ms"
    else
        echo "ERROR:0"
    fi
}

get_process_info() {
    local pid=$(pgrep -f "llmdig" | head -1)
    if [ -n "$pid" ]; then
        local memory_kb=$(ps -o rss= -p "$pid" 2>/dev/null || echo "0")
        local cpu_percent=$(ps -o %cpu= -p "$pid" 2>/dev/null || echo "0")
        local uptime_seconds=$(ps -o etime= -p "$pid" 2>/dev/null || echo "0")
        
        echo "$pid:$memory_kb:$cpu_percent:$uptime_seconds"
    else
        echo "0:0:0:0"
    fi
}

format_uptime() {
    local seconds="$1"
    if [ "$seconds" = "0" ]; then
        echo "Unknown"
    else
        local days=$((seconds / 86400))
        local hours=$(((seconds % 86400) / 3600))
        local minutes=$(((seconds % 3600) / 60))
        
        if [ $days -gt 0 ]; then
            echo "${days}d ${hours}h ${minutes}m"
        elif [ $hours -gt 0 ]; then
            echo "${hours}h ${minutes}m"
        else
            echo "${minutes}m"
        fi
    fi
}

format_memory() {
    local kb="$1"
    if [ "$kb" = "0" ]; then
        echo "Unknown"
    else
        local mb=$((kb / 1024))
        local gb=$((mb / 1024))
        
        if [ $gb -gt 0 ]; then
            echo "${gb}GB"
        else
            echo "${mb}MB"
        fi
    fi
}

get_network_stats() {
    # Get network interface statistics
    local interface="lo"  # Default to loopback
    if command -v ip &> /dev/null; then
        interface=$(ip route | grep default | awk '{print $5}' | head -1)
    fi
    
    if [ -n "$interface" ] && [ -f "/sys/class/net/$interface/statistics/rx_bytes" ]; then
        local rx_bytes=$(cat "/sys/class/net/$interface/statistics/rx_bytes")
        local tx_bytes=$(cat "/sys/class/net/$interface/statistics/tx_bytes")
        echo "$interface:$rx_bytes:$tx_bytes"
    else
        echo "unknown:0:0"
    fi
}

get_system_stats() {
    # CPU usage
    local cpu_usage=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    
    # Memory usage
    local total_mem=$(free | grep Mem | awk '{print $2}')
    local used_mem=$(free | grep Mem | awk '{print $3}')
    local mem_percent=$((used_mem * 100 / total_mem))
    
    # Disk usage
    local disk_usage=$(df / | tail -1 | awk '{print $5}' | cut -d'%' -f1)
    
    echo "$cpu_usage:$mem_percent:$disk_usage"
}

log_metrics() {
    local timestamp="$1"
    local server_status="$2"
    local response_time="$3"
    local process_info="$4"
    local network_stats="$5"
    local system_stats="$6"
    
    # Parse process info
    IFS=':' read -r pid memory_kb cpu_percent uptime_seconds <<< "$process_info"
    
    # Parse network stats
    IFS=':' read -r interface rx_bytes tx_bytes <<< "$network_stats"
    
    # Parse system stats
    IFS=':' read -r cpu_usage mem_percent disk_usage <<< "$system_stats"
    
    # Create JSON metrics
    cat > "$METRICS_FILE" << EOF
{
    "timestamp": "$timestamp",
    "server": {
        "status": "$server_status",
        "response_time_ms": $response_time
    },
    "process": {
        "pid": $pid,
        "memory_kb": $memory_kb,
        "cpu_percent": $cpu_percent,
        "uptime_seconds": $uptime_seconds
    },
    "network": {
        "interface": "$interface",
        "rx_bytes": $rx_bytes,
        "tx_bytes": $tx_bytes
    },
    "system": {
        "cpu_percent": $cpu_usage,
        "memory_percent": $mem_percent,
        "disk_percent": $disk_usage
    }
}
EOF
}

display_dashboard() {
    local timestamp="$1"
    local server_status="$2"
    local response_time="$3"
    local process_info="$4"
    local network_stats="$5"
    local system_stats="$6"
    
    # Clear screen and move to top
    clear
    print_header
    
    # Server Status Section
    print_section "Server Status"
    if [ "$server_status" = "OK" ]; then
        print_status "OK" "Server is responding"
        print_metric "Response Time" "${response_time}ms" "$CYAN"
    else
        print_status "ERROR" "Server is not responding"
    fi
    echo ""
    
    # Process Information Section
    print_section "Process Information"
    IFS=':' read -r pid memory_kb cpu_percent uptime_seconds <<< "$process_info"
    
    if [ "$pid" != "0" ]; then
        print_metric "PID" "$pid" "$CYAN"
        print_metric "Memory Usage" "$(format_memory $memory_kb)" "$CYAN"
        print_metric "CPU Usage" "${cpu_percent}%" "$CYAN"
        print_metric "Uptime" "$(format_uptime $uptime_seconds)" "$CYAN"
    else
        print_status "WARNING" "LLMdig process not found"
    fi
    echo ""
    
    # System Resources Section
    print_section "System Resources"
    IFS=':' read -r cpu_usage mem_percent disk_usage <<< "$system_stats"
    
    print_metric "System CPU" "${cpu_usage}%" "$CYAN"
    print_metric "System Memory" "${mem_percent}%" "$CYAN"
    print_metric "Disk Usage" "${disk_usage}%" "$CYAN"
    echo ""
    
    # Network Statistics Section
    print_section "Network Statistics"
    IFS=':' read -r interface rx_bytes tx_bytes <<< "$network_stats"
    
    if [ "$interface" != "unknown" ]; then
        print_metric "Interface" "$interface" "$CYAN"
        print_metric "RX Bytes" "$rx_bytes" "$CYAN"
        print_metric "TX Bytes" "$tx_bytes" "$CYAN"
    else
        print_status "WARNING" "Network statistics unavailable"
    fi
    echo ""
    
    # Last Update
    print_metric "Last Update" "$timestamp" "$YELLOW"
    echo ""
    echo "Press Ctrl+C to stop monitoring"
}

monitor_loop() {
    local iteration=0
    
    while true; do
        iteration=$((iteration + 1))
        local timestamp=$(get_timestamp)
        
        # Collect metrics
        local server_result=$(check_server_status)
        IFS=':' read -r server_status response_time <<< "$server_result"
        
        local process_info=$(get_process_info)
        local network_stats=$(get_network_stats)
        local system_stats=$(get_system_stats)
        
        # Log metrics
        log_metrics "$timestamp" "$server_status" "$response_time" "$process_info" "$network_stats" "$system_stats"
        
        # Display dashboard
        display_dashboard "$timestamp" "$server_status" "$response_time" "$process_info" "$network_stats" "$system_stats"
        
        # Log to file
        echo "[$timestamp] Status: $server_status, Response: ${response_time}ms, PID: $(echo $process_info | cut -d: -f1)" >> "$LOG_FILE"
        
        # Wait for next iteration
        sleep $MONITOR_INTERVAL
    done
}

show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --host HOST        LLMdig server host (default: localhost)"
    echo "  --port PORT        LLMdig server port (default: 9000)"
    echo "  --interval SECONDS Monitoring interval in seconds (default: 5)"
    echo "  --log-file FILE    Log file path (default: /tmp/llmdig_monitor.log)"
    echo "  --metrics-file FILE Metrics JSON file path (default: /tmp/llmdig_metrics.json)"
    echo "  --help             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                           # Start monitoring with defaults"
    echo "  $0 --host 192.168.1.100     # Monitor remote server"
    echo "  $0 --interval 10            # Update every 10 seconds"
    echo "  $0 --log-file ./monitor.log # Custom log file"
}

cleanup() {
    echo ""
    echo -e "${YELLOW}Stopping LLMdig monitor...${NC}"
    echo "Log file: $LOG_FILE"
    echo "Metrics file: $METRICS_FILE"
    exit 0
}

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
        --interval)
            MONITOR_INTERVAL="$2"
            shift 2
            ;;
        --log-file)
            LOG_FILE="$2"
            shift 2
            ;;
        --metrics-file)
            METRICS_FILE="$2"
            shift 2
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Create log file if it doesn't exist
touch "$LOG_FILE"

# Start monitoring
monitor_loop 