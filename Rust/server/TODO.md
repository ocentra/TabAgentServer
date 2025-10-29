# TabAgent Server TODO

## High Priority

- [ ] **WebRTC Integration**
  - [ ] Wire AppState to WebRTC data channel handler
  - [ ] Implement request routing through WebRTC
  - [ ] Add WebRTC signaling server port configuration
  - [ ] Test WebRTC with real peer connections
  - Status: WebRTC crate exists but not integrated with AppState

- [ ] **Configuration Management**
  - [ ] Add YAML/TOML configuration file support
  - [ ] Environment variable configuration
  - [ ] Configuration validation
  - [ ] Hot-reload support for non-critical configs

- [ ] **Testing**
  - [ ] Add more integration tests
  - [ ] Test each server mode (native, http, webrtc, both, all)
  - [ ] Test graceful shutdown in all modes
  - [ ] Test error recovery and restart
  - [ ] Load testing for HTTP mode
  - [ ] Native messaging protocol compliance tests

## Medium Priority

- [ ] **Security**
  - [ ] Add TLS/HTTPS support for HTTP mode
  - [ ] Add authentication layer (JWT, API keys)
  - [ ] Add authorization/permissions
  - [ ] Rate limiting per transport
  - [ ] Input validation and sanitization

- [ ] **Monitoring & Observability**
  - [ ] Prometheus metrics endpoint
  - [ ] Health check endpoint improvements
  - [ ] Structured logging with correlation IDs
  - [ ] Performance metrics collection
  - [ ] Memory usage tracking

- [ ] **Performance**
  - [ ] Database connection pooling
  - [ ] Request caching layer
  - [ ] Async model loading
  - [ ] Background task queue
  - [ ] Memory-mapped model files

## Low Priority

- [ ] **Documentation**
  - [ ] Add OpenAPI/Swagger UI for HTTP API
  - [ ] Architecture diagrams
  - [ ] Deployment guide
  - [ ] Performance tuning guide
  - [ ] Troubleshooting guide

- [ ] **Developer Experience**
  - [ ] Docker support
  - [ ] Docker Compose for full stack
  - [ ] Development mode with hot-reload
  - [ ] Better error messages
  - [ ] CLI improvements

- [ ] **Features**
  - [ ] Multi-user support
  - [ ] Session management
  - [ ] Request queuing and prioritization
  - [ ] Model preloading on startup
  - [ ] Automatic model downloads
  - [ ] Model versioning

## Technical Debt

- [ ] Remove Python dependencies completely
- [ ] Clean up unused imports across codebase
- [ ] Add #[deny(missing_docs)] to enforce documentation
- [ ] Standardize error types across crates
- [ ] Add benchmark suite
- [ ] CI/CD pipeline improvements

## Completed ✅

- [x] Basic server structure with multiple modes
- [x] AppState integration
- [x] Native messaging support
- [x] HTTP API support
- [x] Shared state architecture
- [x] CLI argument parsing
- [x] Tracing/logging setup
- [x] Basic error handling
- [x] Integration tests for native-messaging
- [x] Integration tests for API
- [x] Linter error fixes in main.rs
- [x] README documentation
- [x] TODO tracking

## Notes

- WebRTC is the main blocker for "All" mode being fully functional
- Consider splitting server modes into separate binaries for smaller deployments
- Native messaging is production-ready
- HTTP API is production-ready
- Database abstraction could be improved for testing

