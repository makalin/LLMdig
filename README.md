# LLMdig — LLM over DNS 🔍🧠

LLMdig is a **DNS TXT server** that lets you query a large language model (LLM) through `dig` commands — just like:

```bash
dig @llm.pieter.com -p 9000 "what is the meaning of life" TXT +short
````

Yes, it responds via a **DNS TXT record** with an LLM-generated answer. This means you can run LLM queries from places where only DNS traffic is allowed or just for the fun of it.

---

## 🚀 Features

* 📡 **LLM-over-DNS** communication
* ⚙️ Runs a real DNS server on custom port (9000)
* 💬 LLM replies are generated on-demand (OpenAI or local model)
* ⏱️ Minimal latency via async Rust stack
* 🔐 Rate-limiting and input sanitization built-in

---

## 📦 Installation

### Prerequisites

* Rust ([https://rustup.rs](https://rustup.rs))
* An OpenAI API key (or configure a local LLM endpoint)

### Clone & Run

```bash
git clone https://github.com/makalin/LLMdig.git
cd LLMdig
cargo run --release
```

By default, the server will run on `0.0.0.0:9000`.

---

## ⚙️ Configuration

Set environment variables:

```bash
OPENAI_API_KEY=sk-xxxxx
PORT=9000
MODEL=gpt-3.5-turbo
```

You can switch to a local model like Ollama:

```bash
LLM_BACKEND=http://localhost:11434/api/generate
```

---

## 🧪 Example Query

```bash
dig @localhost -p 9000 "who built the pyramids?" TXT +short
```

**Output:**

```
"The Great Pyramids were constructed by skilled workers under Pharaoh Khufu's reign using limestone and granite."
```

---

## 🛠️ Stack

* 🦀 Rust
* 🧠 OpenAI API or Ollama
* 🌐 trust-dns-server
* 🔄 tokio (async runtime)

---

## ✨ Use Cases

* Command-line LLM access
* Offline/air-gapped LLM gateway
* DNS-only networks
* Novelty & curiosity projects

---

## 📜 License

MIT — use it freely, vibe responsibly.
