# Virtual Engineer Setup Guide

## Overview

The Virtual Engineer is an AI chat assistant that helps users understand their quality control analysis results. It runs as a separate service alongside the main server.

## Components

- **engineer/** - Rust WebSocket service (port 8082)
- **Ollama** - Local LLM inference server
- **UI Chat Component** - Integrated in the sidebar

## VPS Setup Instructions

### 1. Install Ollama

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

### 2. Pull the LLM Model

For 8GB RAM, use Phi-3 mini (recommended):
```bash
ollama pull phi3:mini
```

Alternative lightweight models:
```bash
# Even smaller
ollama pull qwen2.5:1.5b

# Better quality but uses more RAM (~6GB)
ollama pull mistral:7b-instruct-q4_0
```

### 3. Configure Ollama as a Service

Ollama installs its own systemd service. Verify it's running:
```bash
sudo systemctl status ollama
```

### 4. Build Virtual Engineer

On VPS:
```bash
cd /home/vp/quality_control_room/engineer
cargo build --release
```

### 5. Install Systemd Service

```bash
sudo cp /home/vp/quality_control_room/systemd/virtual-engineer.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable virtual-engineer
sudo systemctl start virtual-engineer
```

### 6. Open Firewall Port

```bash
sudo ufw allow 8082/tcp
```

### 7. Verify Services

```bash
# Check Ollama
curl http://localhost:11434/api/tags

# Check Virtual Engineer  
curl https://quality-control.io:8082/health
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENGINEER_PORT` | 8082 | WebSocket server port |
| `ENGINEER_MODEL` | phi3:mini | Ollama model to use |

## Memory Usage

| Component | Estimated RAM |
|-----------|---------------|
| Ollama + phi3:mini | ~3-4 GB |
| Virtual Engineer service | ~50 MB |
| Quality Server | ~100 MB |
| **Total** | ~4-5 GB |

## Troubleshooting

### Ollama not responding
```bash
sudo systemctl restart ollama
journalctl -u ollama -f
```

### Virtual Engineer connection failed
```bash
sudo systemctl status virtual-engineer
journalctl -u virtual-engineer -f
```

### Model too slow
Try a smaller model:
```bash
export ENGINEER_MODEL=qwen2.5:1.5b
sudo systemctl restart virtual-engineer
```

## API Protocol

WebSocket messages are JSON:

**Client → Server:**
```json
{"command": "status"}
{"command": "chat", "message": "What do my results mean?", "context": {...}}
```

**Server → Client:**
```json
{"command": "status", "available": true}
{"command": "chunk", "content": "The ", "done": false}
{"command": "done", "done": true}
{"command": "error", "error": "message", "done": true}
```
