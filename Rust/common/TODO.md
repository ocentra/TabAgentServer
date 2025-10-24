# Common Crate - TODO

## ✅ Completed

### Phase 1: Foundation
- [x] Type aliases (NodeId, EdgeId, EmbeddingId)
- [x] Error types (DbError, DbResult)
- [x] JSON metadata serialization helper
- [x] Core models (Node, Edge, Embedding)
- [x] Basic node types (Chat, Message, Summary, Attachment, Entity)

### Phase 1.5: MIA Data Model
- [x] WebSearch node type
- [x] ScrapedPage node type
- [x] Bookmark node type
- [x] ImageMetadata node type
- [x] AudioTranscript node type
- [x] ModelInfo node type
- [x] Enhanced Attachment with extracted_text and detected_objects
- [x] Updated Message model (text_content, attachment_ids)

### Testing
- [x] 7 doc tests passing
- [x] 8 unit tests covering newtype wrappers, error types, and platform functionality
- [x] All models serialize/deserialize correctly

## 🔄 In Progress

_Nothing currently in progress_

## 📋 Pending

### Future Node Types (As Needed)
- [ ] Credential (encrypted passwords, API keys)
- [ ] CalendarEvent (meetings, reminders)
- [ ] Location (GPS, place names)
- [ ] Contact (people, organizations)
- [ ] Video (metadata, transcripts)
- [ ] Document (PDFs, Word docs)
- [ ] AppState (UI state persistence)
- [ ] Habit (user patterns)
- [ ] Pattern (recognized behaviors)
- [ ] Context (situational metadata)

### Enhancements
- [ ] Timestamp validation helpers
- [ ] Node validation methods
- [ ] Schema versioning for migrations

## 🚫 Blockers

_No current blockers_

## 📊 Progress

- **Phase 1**: ✅ 100% Complete
- **Phase 1.5**: ✅ 100% Complete
- **Testing**: ✅ 100% Complete
- **Overall**: **STABLE** - Ready for production use