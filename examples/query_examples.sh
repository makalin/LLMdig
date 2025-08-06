#!/bin/bash

# LLMdig Query Examples
# This script demonstrates various ways to query the LLMdig DNS server

# Configuration
LLMDIG_HOST="localhost"
LLMDIG_PORT="9000"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 LLMdig Query Examples${NC}"
echo "================================"
echo ""

# Function to run a query and display results
run_query() {
    local query="$1"
    local description="$2"
    
    echo -e "${YELLOW}Query:${NC} $description"
    echo -e "${GREEN}Domain:${NC} $query"
    echo -e "${GREEN}Command:${NC} dig @$LLMDIG_HOST -p $LLMDIG_PORT '$query' TXT +short"
    echo ""
    
    # Run the query
    result=$(dig @$LLMDIG_HOST -p $LLMDIG_PORT "$query" TXT +short 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$result" ]; then
        echo -e "${GREEN}Response:${NC}"
        echo "$result"
    else
        echo -e "${YELLOW}No response or error${NC}"
    fi
    
    echo ""
    echo "----------------------------------------"
    echo ""
}

# Check if dig is available
if ! command -v dig &> /dev/null; then
    echo "Error: 'dig' command not found. Please install bind-utils or dnsutils."
    exit 1
fi

# Check if server is running
if ! dig @$LLMDIG_HOST -p $LLMDIG_PORT "test.com" TXT +short &> /dev/null; then
    echo "Warning: LLMdig server doesn't seem to be running on $LLMDIG_HOST:$LLMDIG_PORT"
    echo "Start the server with: cargo run --release"
    echo ""
fi

# Basic questions
run_query "what.is.the.weather.com" "Basic weather question"
run_query "how.many.stars.are.there.com" "Astronomy question"
run_query "what.is.the.capital.of.france.com" "Geography question"

# Questions with hyphens and underscores
run_query "hello-world.example.com" "Question with hyphens"
run_query "how_are_you_today.example.com" "Question with underscores"

# Complex questions
run_query "what.is.the.meaning.of.life.com" "Philosophical question"
run_query "how.do.i.cook.pasta.example.com" "Cooking question"
run_query "what.is.the.best.programming.language.com" "Technology question"

# Short questions
run_query "hello.example.com" "Short greeting"
run_query "help.example.com" "Help request"

# Questions with numbers
run_query "what.is.2.plus.2.example.com" "Math question"
run_query "how.many.days.in.a.year.example.com" "Question with numbers"

# Health check
echo -e "${YELLOW}Health Check:${NC}"
echo -e "${GREEN}Command:${NC} dig @$LLMDIG_HOST -p $LLMDIG_PORT 'health.check' TXT +short"
echo ""

health_result=$(dig @$LLMDIG_HOST -p $LLMDIG_PORT "health.check" TXT +short 2>/dev/null)
if [ $? -eq 0 ] && [ -n "$health_result" ]; then
    echo -e "${GREEN}Health Status:${NC} $health_result"
else
    echo -e "${YELLOW}Health check failed${NC}"
fi

echo ""
echo "========================================"
echo -e "${BLUE}Usage Tips:${NC}"
echo "1. Replace dots with spaces to form questions"
echo "2. Use hyphens or underscores for multi-word concepts"
echo "3. Keep questions concise for better responses"
echo "4. The server caches responses for 5 minutes"
echo "5. Rate limiting applies per client IP"
echo ""
echo -e "${BLUE}Server Configuration:${NC}"
echo "- Default port: 9000"
echo "- Supports OpenAI, Ollama, and custom backends"
echo "- Rate limited to 60 requests/minute by default"
echo "- Responses truncated to fit DNS TXT records" 