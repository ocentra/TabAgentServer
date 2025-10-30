# Core - TODO

## Current State

✅ **Implemented**:
- RustFileProvider with gRPC streaming
- StreamHandler with multiple source support
- VideoFrame conversion utilities
- WebRTC stream protocol decoding

⚙️ **Tested**:
- File provider integration
- Stream format conversion
- Camera capture
- Screen capture

---

## Planned Features

### High Priority

- [ ] **Audio Stream Support**
  - AudioFrame message type
  - Microphone capture
  - Audio file reading
  - Format conversion (PCM, MP3, etc.)

- [ ] **H264/VP8 Decoding**
  - Use ffmpeg-python for video codec support
  - Hardware-accelerated decoding
  - Real-time decompression

- [ ] **Stream Buffering**
  - Frame queue for smooth playback
  - Adaptive buffering based on latency
  - Dropped frame handling

### Medium Priority

- [ ] **Advanced File Provider**
  - Cache recently fetched files locally
  - Parallel chunk downloads
  - Resume interrupted transfers
  - Checksum verification

- [ ] **Stream Recording**
  - Save streams to video files
  - Timestamp synchronization
  - Metadata embedding

- [ ] **Multi-stream Support**
  - Handle multiple concurrent streams
  - Stream multiplexing
  - Per-stream configuration

### Low Priority

- [ ] **Stream Analytics**
  - FPS monitoring
  - Latency tracking
  - Bandwidth usage
  - Frame drop detection

- [ ] **Format Auto-detection**
  - Detect video format from headers
  - Automatic codec selection
  - Fallback handling

---

## Known Issues

- **H264/VP8 streams**: Not yet decoded, passed as raw bytes
- **Audio streams**: Not yet supported (video only)
- **mss library**: Optional dependency, screen capture may fail if not installed

---

## Notes

- RustFileProvider is critical for model loading - prevents duplicate downloads
- StreamHandler is performance-sensitive - avoid blocking operations
- All async generators must handle cleanup properly (close resources)

---

## Dependencies to Add

```txt
# For advanced features
ffmpeg-python==0.2.0  # Video codec support
pyaudio==0.2.13       # Audio capture
sounddevice==0.4.6    # Alternative audio I/O
```

