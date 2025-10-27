# 🚧 TabAgent WebRTC - TODO

**Mission**: Provide a complete alternative to Native Messaging using WebRTC data channels, enabling Chrome extensions to communicate with TabAgent server without native host installation.

---

## 📊 Current Status: **0% Complete**

```
┌─────────────────────────────────────────────────────┐
│ Phase 1: Core Infrastructure        [░░░░░░░░░░] 0% │
│ Phase 2: Signaling                   [░░░░░░░░░░] 0% │
│ Phase 3: Data Channels               [░░░░░░░░░░] 0% │
│ Phase 4: Integration                 [░░░░░░░░░░] 0% │
│ Phase 5: Testing                     [░░░░░░░░░░] 0% │
│ Phase 6: Chrome Extension            [░░░░░░░░░░] 0% │
└─────────────────────────────────────────────────────┘
```

---

## 🎯 Phase 1: Core Infrastructure [0/6]

### File Structure
- [ ] `src/lib.rs` - Public API exports
- [ ] `src/manager.rs` - WebRtcManager (session orchestrator)
- [ ] `src/session.rs` - Session state management
- [ ] `src/config.rs` - Configuration types
- [ ] `src/error.rs` - Error types
- [ ] `src/types.rs` - WebRTC-specific types

### Dependencies
- [ ] Add `webrtc` crate (0.11)
- [ ] Add `tokio` for async
- [ ] Add `tabagent-values` for Request/Response
- [ ] Verify all dependencies compile

---

## 🎯 Phase 2: Signaling Implementation [0/8]

### Session Creation
- [ ] `WebRtcManager::create_offer()` - Generate SDP offer
- [ ] `WebRtcSession` struct - Store session state
- [ ] UUID-based session IDs
- [ ] Store offer SDP in session

### Answer Handling
- [ ] `WebRtcManager::submit_answer()` - Store answer SDP
- [ ] Validate session exists
- [ ] Update session state to IceGathering
- [ ] Store answer for later retrieval

### ICE Candidate Management
- [ ] `WebRtcManager::add_ice_candidate()` - Add candidate
- [ ] Store ICE candidates in session
- [ ] Forward candidates to peer connection
- [ ] Handle ICE completion

### Session Queries
- [ ] `WebRtcManager::get_session()` - Get session by ID
- [ ] `WebRtcManager::list_sessions()` - List all sessions
- [ ] Return session state, timestamps
- [ ] Filter by state (connected, waiting, etc.)

### Cleanup
- [ ] Background task for stale session cleanup
- [ ] Remove sessions inactive for > 5 minutes
- [ ] Close data channels gracefully
- [ ] Log cleanup events

---

## 🎯 Phase 3: Data Channel Implementation [0/10]

### Channel Setup
- [ ] Create `DataChannelHandler` struct
- [ ] Configure ordered, reliable delivery
- [ ] Set binary message support
- [ ] Add message size limits (1MB default)

### Message Routing
- [ ] Parse incoming JSON as `RequestValue`
- [ ] Validate request format
- [ ] Route to `tabagent-server::Handler`
- [ ] Serialize `ResponseValue` back

### Request Handlers (via server::Handler)
- [ ] Chat requests → `handle_chat()`
- [ ] Model loading → `handle_load_model()`
- [ ] Embeddings → `handle_embeddings()`
- [ ] RAG → `handle_rag()`
- [ ] All 36 API routes supported!

### Streaming Support
- [ ] Implement streaming callback for chat
- [ ] Send partial tokens over data channel
- [ ] Handle backpressure (slow client)
- [ ] Final response with full text

### Error Handling
- [ ] Convert `anyhow::Error` to JSON error response
- [ ] Send error messages over data channel
- [ ] Log all errors with session ID
- [ ] Close channel on fatal errors

### Server-Initiated Push
- [ ] Model loading progress events
- [ ] System notifications
- [ ] Resource usage alerts
- [ ] Bidirectional event system

---

## 🎯 Phase 4: Integration with Server [0/6]

### AppState Integration
- [ ] Add `webrtc: Arc<WebRtcManager>` to `AppState`
- [ ] Initialize WebRTC manager in `main.rs`
- [ ] Pass handler reference to data channel
- [ ] Share state between HTTP and WebRTC

### API Route Integration
- [ ] Wire `/v1/webrtc/offer` to `manager.create_offer()`
- [ ] Wire `/v1/webrtc/answer` to `manager.submit_answer()`
- [ ] Wire `/v1/webrtc/ice` to `manager.add_ice_candidate()`
- [ ] Wire `/v1/webrtc/session` to `manager.get_session()`

### Handler Integration
- [ ] Reuse `server::Handler::handle_request()`
- [ ] Pass same `AppState` to WebRTC messages
- [ ] Share model cache, database, hardware info
- [ ] Same error propagation as HTTP API

### Configuration
- [ ] Load WebRTC config from environment
- [ ] Support STUN/TURN server URLs
- [ ] Configure timeouts, limits
- [ ] Add feature flags (webrtc-enabled)

---

## 🎯 Phase 5: Testing [0/8]

### Unit Tests
- [ ] Test session creation and storage
- [ ] Test ICE candidate handling
- [ ] Test message parsing (JSON → RequestValue)
- [ ] Test response serialization (ResponseValue → JSON)

### Integration Tests
- [ ] Full signaling flow (offer → answer → ICE)
- [ ] Data channel message exchange
- [ ] Multiple concurrent sessions
- [ ] Session cleanup and timeout

### Load Tests
- [ ] 100 concurrent sessions
- [ ] 1000 messages/second throughput
- [ ] Memory usage under load
- [ ] CPU usage under load

### Error Tests
- [ ] Invalid SDP handling
- [ ] Malformed JSON messages
- [ ] Session not found
- [ ] Data channel disconnect

---

## 🎯 Phase 6: Chrome Extension Support [0/6]

### Client Library (`extension/src/webrtc-client.ts`)
- [ ] `TabAgentWebRTC` class
- [ ] `connect()` - Establish connection
- [ ] `sendRequest()` - Send RequestValue
- [ ] `onResponse()` - Receive ResponseValue
- [ ] `disconnect()` - Close connection
- [ ] Auto-reconnect on failure

### Extension Integration
- [ ] Background service worker WebRTC manager
- [ ] Content script message forwarding
- [ ] UI preference (WebRTC vs Native Messaging)
- [ ] Fallback to Native Messaging if WebRTC fails

### Documentation
- [ ] Extension setup guide
- [ ] WebRTC vs Native Messaging comparison
- [ ] Troubleshooting guide
- [ ] Network requirements (firewall, CORS)

---

## 📋 Feature Parity Checklist

### Supported Request Types (All 36!)

**Chat & Generation**
- [ ] Chat (streaming + non-streaming)
- [ ] Responses (session-based chat)
- [ ] Stop generation
- [ ] Get halt status

**Model Management**
- [ ] List models
- [ ] Load model
- [ ] Unload model
- [ ] Get model info
- [ ] Pull model (download)
- [ ] Delete model

**Embeddings & RAG**
- [ ] Generate embeddings
- [ ] RAG query
- [ ] Rerank documents
- [ ] Add document
- [ ] Delete document
- [ ] List collections
- [ ] Clear collection

**Session History**
- [ ] Get history
- [ ] Save message

**System & Resources**
- [ ] Health check
- [ ] System info
- [ ] Get stats
- [ ] Get resources
- [ ] Estimate memory
- [ ] Check compatibility
- [ ] Get recipes
- [ ] Get registered models
- [ ] List loaded models
- [ ] Select model

**Parameters**
- [ ] Get params
- [ ] Set params

---

## 🔧 Technical Debt

### Performance Optimization
- [ ] Connection pooling for data channels
- [ ] Message batching for high-throughput
- [ ] Zero-copy binary transfer
- [ ] WebAssembly for extension client

### Monitoring
- [ ] Add metrics (Prometheus)
- [ ] Session duration tracking
- [ ] Message throughput stats
- [ ] Error rate monitoring

### Documentation
- [ ] API documentation (rustdoc)
- [ ] Architecture diagrams
- [ ] Sequence diagrams for flows
- [ ] Performance benchmarks

---

## 🚀 Future Enhancements

### Advanced Features
- [ ] Multiple data channels per session
- [ ] Binary model transfer (gguf files)
- [ ] Screen sharing integration
- [ ] Voice/video calls (for voice assistants)

### Enterprise Features
- [ ] JWT authentication for signaling
- [ ] Rate limiting per session
- [ ] Audit logging
- [ ] Multi-tenant support

### Developer Experience
- [ ] CLI tool for WebRTC testing
- [ ] Mock WebRTC server for extension dev
- [ ] Chrome DevTools panel integration
- [ ] Performance profiler

---

## 🐛 Known Issues

None yet - we're just getting started! 🎉

---

## 📝 Notes

### Why WebRTC Over Native Messaging?

**Advantages:**
- No native host installation required
- Works in sandboxed environments (Chrome OS, enterprise)
- Server can push updates to client
- Multi-tab support out of the box

**Disadvantages:**
- Slightly higher latency (~5-10ms vs 1-2ms)
- Requires network access (localhost or remote)
- More complex setup (signaling)

### Design Decisions

1. **REST API for Signaling** - HTTP is simpler than WebSocket for offer/answer
2. **Data Channels for Messages** - Lower latency than HTTP polling
3. **Reuse tabagent-values** - Same types as HTTP API and Native Messaging
4. **Shared Handler** - All three entry points use same backend logic

### Migration Path

```
Phase 1 (Current): Native Messaging only
         ↓
Phase 2: Add WebRTC as alternative
         ↓
Phase 3: Default to WebRTC, fallback to Native Messaging
         ↓
Phase 4: WebRTC only (native messaging deprecated)
```

---

## 🤝 Contributing

See main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

**Last Updated**: 2025-10-27  
**Status**: Planning → Implementation  
**Target Completion**: Q1 2026

