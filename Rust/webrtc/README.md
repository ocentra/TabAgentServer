# 🌐 TabAgent WebRTC

**Enterprise-grade WebRTC signaling and data channel handler for TabAgent server.**

This crate provides a complete alternative to Native Messaging for Chrome extensions, enabling real-time bidirectional communication over WebRTC data channels.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Features](#features)
- [Usage](#usage)
- [WebRTC Flow](#webrtc-flow)
- [Session Management](#session-management)
- [Data Channel Protocol](#data-channel-protocol)
- [Security](#security)
- [Comparison: WebRTC vs Native Messaging](#comparison-webrtc-vs-native-messaging)

---

## 🎯 Overview

`tabagent-webrtc` enables Chrome extensions to communicate with TabAgent server using **WebRTC data channels** instead of Chrome Native Messaging. This provides:

- ✅ **No Native Host Installation Required** - Works directly from the browser
- ✅ **Bidirectional Real-Time Communication** - Server can push to client
- ✅ **Same Protocol as Native Messaging** - Uses `tabagent-values` Request/Response
- ✅ **Multi-Session Support** - Handle multiple Chrome tabs simultaneously
- ✅ **Automatic Reconnection** - Resilient to network issues

### When to Use WebRTC vs Native Messaging

| Feature | WebRTC | Native Messaging |
|---------|--------|------------------|
| Installation | None (browser-only) | Requires native host install |
| Performance | ~5-10ms latency | ~1-2ms latency |
| Server Push | ✅ Yes | ❌ No (request-only) |
| Multi-tab | ✅ Yes | ⚠️ Requires port coordination |
| Firewall | May require port 8080 open | Works locally |
| Use Case | Production deployments | Local development |

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Chrome Extension                         │
│                                                                 │
│  ┌────────────┐         ┌─────────────┐        ┌─────────────┐ │
│  │  Content   │ ──────→ │  Background │ ────→  │   WebRTC    │ │
│  │   Script   │         │   Service   │        │   Client    │ │
│  └────────────┘         └─────────────┘        └──────┬──────┘ │
└────────────────────────────────────────────────────────┼────────┘
                                                         │
                                    REST API (signaling) │
                                                         ↓
┌─────────────────────────────────────────────────────────────────┐
│                      TabAgent Server (Rust)                     │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              tabagent-api (HTTP Routes)                  │  │
│  │  POST /v1/webrtc/offer    - Create session              │  │
│  │  POST /v1/webrtc/answer   - Submit answer               │  │
│  │  POST /v1/webrtc/ice      - Add ICE candidate           │  │
│  │  GET  /v1/webrtc/session  - Get session info            │  │
│  └──────────────────────┬───────────────────────────────────┘  │
│                         │                                       │
│                         ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │           tabagent-webrtc (This Crate)                   │  │
│  │                                                          │  │
│  │  ┌────────────────┐  ┌─────────────┐  ┌──────────────┐ │  │
│  │  │    Session     │  │    Data     │  │   Message    │ │  │
│  │  │    Manager     │  │   Channel   │  │   Router     │ │  │
│  │  │                │  │   Handler   │  │              │ │  │
│  │  │ - Offer/Answer │  │ - Binary    │  │ - Request    │ │  │
│  │  │ - ICE handling │  │ - JSON msgs │  │ - Response   │ │  │
│  │  │ - Cleanup      │  │ - Streaming │  │ - Events     │ │  │
│  │  └────────────────┘  └─────────────┘  └──────────────┘ │  │
│  └──────────────────────┬───────────────────────────────────┘  │
│                         │                                       │
│                         ↓                                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │            tabagent-server (Handler)                     │  │
│  │  - Model loading, inference, embeddings, RAG...         │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

---

## ✨ Features

### Signaling (REST API)
- **SDP Offer/Answer Exchange** - WebRTC handshake via HTTP
- **ICE Candidate Trickling** - Progressive connectivity establishment
- **Session State Management** - Track connection lifecycle

### Data Channels
- **Request/Response Protocol** - Same as Native Messaging
- **Binary Message Support** - Efficient model data transfer
- **Streaming Support** - Real-time token streaming for chat
- **Server-Initiated Push** - Model loading progress, events

### Session Management
- **Multi-Session Support** - Handle multiple Chrome tabs
- **Automatic Cleanup** - Remove stale sessions after timeout
- **Reconnection Handling** - Graceful session resumption

### Integration
- **Uses `tabagent-values`** - Shared Request/Response types
- **Calls `tabagent-server` Handler** - Same backend as HTTP API
- **Trait-Based Design** - Extensible and testable

---

## 🚀 Usage

### Server-Side Integration

```rust
use tabagent_webrtc::{WebRtcManager, WebRtcConfig};
use tabagent_server::AppState;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create WebRTC manager
    let config = WebRtcConfig::default();
    let webrtc_manager = Arc::new(WebRtcManager::new(config));
    
    // Add to app state
    let state = Arc::new(AppState {
        webrtc: webrtc_manager.clone(),
        // ... other fields
    });
    
    // API routes will use state.webrtc for signaling
    // Data channel messages automatically route to handler
}
```

### API Route Handlers

```rust
// In tabagent-api/src/routes/webrtc.rs
async fn create_offer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateOfferRequest>,
) -> Result<Json<CreateOfferResponse>, ApiError> {
    // Delegate to WebRTC manager
    let session = state.webrtc.create_offer().await?;
    
    Ok(Json(CreateOfferResponse {
        session_id: session.id,
        offer_sdp: session.offer_sdp,
    }))
}
```

### Chrome Extension Client

```javascript
// In extension background service worker
class TabAgentWebRTC {
  async connect() {
    // 1. Create RTCPeerConnection
    this.pc = new RTCPeerConnection({
      iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
    });
    
    // 2. Create data channel
    this.channel = this.pc.createDataChannel('tabagent', {
      ordered: true,
      maxRetransmits: 3
    });
    
    // 3. Get offer from server
    const { sessionId, offerSdp } = await fetch('http://localhost:8080/v1/webrtc/offer', {
      method: 'POST',
      body: JSON.stringify({ clientId: crypto.randomUUID() })
    }).then(r => r.json());
    
    // 4. Set remote description
    await this.pc.setRemoteDescription({ type: 'offer', sdp: offerSdp });
    
    // 5. Create answer
    const answer = await this.pc.createAnswer();
    await this.pc.setLocalDescription(answer);
    
    // 6. Send answer to server
    await fetch(`http://localhost:8080/v1/webrtc/answer/${sessionId}`, {
      method: 'POST',
      body: JSON.stringify({ answerSdp: answer.sdp })
    });
    
    // 7. Handle ICE candidates
    this.pc.onicecandidate = async (event) => {
      if (event.candidate) {
        await fetch(`http://localhost:8080/v1/webrtc/ice/${sessionId}`, {
          method: 'POST',
          body: JSON.stringify({ candidate: event.candidate })
        });
      }
    };
    
    // 8. Data channel ready!
    this.channel.onopen = () => console.log('WebRTC connected!');
    this.channel.onmessage = (event) => this.handleMessage(event.data);
  }
  
  async sendRequest(request) {
    const message = JSON.stringify(request);
    this.channel.send(message);
  }
  
  handleMessage(data) {
    const response = JSON.parse(data);
    // Handle ResponseValue from server
  }
}
```

---

## 🔄 WebRTC Flow

### 1. Connection Establishment (Signaling)

```
Chrome Extension                    TabAgent Server
     │                                    │
     │  POST /v1/webrtc/offer            │
     ├──────────────────────────────────►│
     │  { clientId: "..." }              │
     │                                    │  Create session
     │                                    │  Generate SDP offer
     │  ◄──────────────────────────────┤
     │  { sessionId, offerSdp }          │
     │                                    │
     │  setRemoteDescription(offer)      │
     │  createAnswer()                   │
     │                                    │
     │  POST /v1/webrtc/answer           │
     ├──────────────────────────────────►│
     │  { answerSdp }                    │
     │                                    │  Store answer
     │                                    │  Start ICE
     │  ◄──────────────────────────────┤
     │  { success: true }                │
     │                                    │
     │  POST /v1/webrtc/ice              │
     ├──────────────────────────────────►│
     │  { candidate }                    │  (Multiple times)
     │                                    │
     │  ◄──────────────────────────────┤
     │  { success: true }                │
     │                                    │
     │      ═══ P2P Connection ═══       │
     │  ◄═══════════════════════════════►│
     │      Data Channel Open            │
```

### 2. Message Exchange (Data Channel)

```
Chrome Extension                    TabAgent Server
     │                                    │
     │  RequestValue::chat(...)          │
     ├──────────────────────────────────►│
     │                                    │  Parse JSON
     │                                    │  Call handler.rs
     │                                    │  Execute inference
     │  ◄──────────────────────────────┤
     │  ResponseValue::chat(...)         │
     │                                    │
     │  RequestValue::load_model(...)    │
     ├──────────────────────────────────►│
     │                                    │  Start model load
     │  ◄──────────────────────────────┤
     │  Event::ModelLoadingProgress      │  (Streaming)
     │  ◄──────────────────────────────┤
     │  Event::ModelLoadingProgress      │
     │  ◄──────────────────────────────┤
     │  ResponseValue::model_loaded      │
```

---

## 📦 Session Management

### Session States

```rust
pub enum SessionState {
    /// Offer created, waiting for answer
    WaitingForAnswer,
    
    /// Answer received, ICE in progress
    IceGathering,
    
    /// Data channel connected
    Connected,
    
    /// Connection failed or closed
    Disconnected,
}
```

### Session Lifecycle

```rust
pub struct WebRtcSession {
    pub id: String,
    pub client_id: String,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    
    // WebRTC state
    pub offer_sdp: String,
    pub answer_sdp: Option<String>,
    pub ice_candidates: Vec<IceCandidate>,
    
    // Data channel
    pub data_channel: Option<Arc<DataChannel>>,
}
```

### Cleanup Policy

- **Timeout**: Sessions inactive for 5 minutes are removed
- **Max Sessions**: Up to 100 concurrent sessions
- **Graceful Shutdown**: Close all data channels on server stop

---

## 📡 Data Channel Protocol

### Message Format

All messages use `tabagent-values::RequestValue` and `ResponseValue`:

```rust
// Client → Server
{
  "request_type": "Chat",
  "model": "phi-3-mini",
  "messages": [...],
  "temperature": 0.7
}

// Server → Client
{
  "response_type": "ChatComplete",
  "text": "Hello! How can I help?",
  "model": "phi-3-mini",
  "usage": { "prompt_tokens": 10, "completion_tokens": 5 }
}
```

### Supported Request Types

**All 36 API routes are supported over WebRTC data channels!**

- Chat, Embeddings, RAG, Rerank
- Model Management (load, unload, list)
- Session History (get, save)
- System Info, Stats, Resources
- Generation Control (stop, halt status)
- Parameters (get, set)

---

## 🔒 Security

### HTTPS/WSS in Production

```rust
let config = WebRtcConfig {
    // Use HTTPS for signaling in production
    signaling_url: "https://api.tabagent.com".to_string(),
    
    // Require authentication token
    require_auth: true,
    
    // Limit sessions per IP
    max_sessions_per_ip: 5,
    
    ..Default::default()
};
```

### Authentication

```javascript
// Extension sends API key in initial offer
const response = await fetch('/v1/webrtc/offer', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY'
  },
  body: JSON.stringify({ clientId: '...' })
});
```

### Rate Limiting

- **Signaling**: 10 requests/minute per IP
- **Data Channel**: 100 messages/second per session
- **Automatic Throttling**: Slow down abusive clients

---

## 📊 Comparison: WebRTC vs Native Messaging

### Performance

| Metric | WebRTC | Native Messaging |
|--------|--------|------------------|
| Latency | 5-10ms | 1-2ms |
| Throughput | 10 MB/s | 50 MB/s |
| CPU Overhead | ~2% | ~0.5% |

### Deployment

| Aspect | WebRTC | Native Messaging |
|--------|--------|------------------|
| User Setup | None | Install native host |
| Permissions | Network only | Full system access |
| Updates | Server-side | Client-side manifest |
| Debugging | Chrome DevTools | stdout logs |

### Recommendation

- **Production/SaaS**: Use WebRTC (no install required)
- **Enterprise/Local**: Use Native Messaging (better performance)
- **Hybrid**: Support both! (This architecture does)

---

## 🧪 Testing

```bash
# Run unit tests
cd Rust/webrtc
cargo test

# Run integration tests
cargo test --test webrtc_integration

# Test with Chrome extension
cd Extension
npm run build
# Load unpacked extension, test WebRTC connection
```

---

## 📚 Related Crates

- [`tabagent-api`](../api/README.md) - HTTP API routes
- [`tabagent-server`](../server/README.md) - Main server logic
- [`tabagent-values`](../values/README.md) - Shared types
- [`tabagent-native-handler`](../native-handler/README.md) - GGUF/BitNet backend

---

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for development guidelines.

## 📄 License

MIT - See [LICENSE](../../LICENSE) for details.

