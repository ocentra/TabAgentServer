# Native Messaging TODO

## ✅ TIER 0 - Architecture Migration (COMPLETED!)

- ✅ Migrated to `common::backend::AppStateProvider`
- ✅ Wired to real `tabagent-server::AppState`
- ✅ Fixed all 4 failing tests:
  - ✅ `router::tests::test_message_dispatch_success`
  - ✅ `router::tests::test_message_dispatch_route_not_found`
  - ✅ `routes::stats::tests::test_stats`
  - ✅ `routes::stats::tests::test_has_test_cases`
- ✅ All 42 unit tests passing

## 🔥 CRITICAL - Route Duplication Cleanup (DO AFTER ALL 3 ENTRY POINTS MIGRATE)

**IMPORTANT:** The `src/routes/` folder contains ~2000 lines of duplicate code that mirrors `api/src/routes/`.

This was acceptable during the migration, but **MUST BE DELETED** once all three entry points are migrated to `common` traits.

**Post-Migration Plan:**
1. Delete entire `native-messaging/src/routes/` folder
2. Delete `native-messaging/src/route_trait.rs`
3. Delete `native-messaging/src/middleware.rs`
4. Replace with a single thin dispatcher:
   ```rust
   // In main.rs or handler.rs
   async fn dispatch_message(msg: IncomingMessage, state: Arc<dyn AppStateProvider>) 
       -> OutgoingMessage 
   {
       let request = RequestValue::from_json(&msg.payload)?;
       let response = state.handle_request(request).await?;
       OutgoingMessage::success(msg.request_id, response.to_json_value())
   }
   ```

**Why this is OK for now:**
- Native messaging needs protocol handling (stdin/stdout, length-prefixed frames)
- The route files provide typed validation at the native messaging layer
- After migration, validation moves to `common` and we just dispatch to backend

**Size Reduction:**
- Current: ~3500 lines (routes + route_trait + middleware)
- After cleanup: ~50 lines (just dispatcher)
- **97% code reduction!**

## TIER 1 - Critical Features (Pending)

### HuggingFace Authentication
- [ ] Add HF token routes (set_token, get_token, clear_token)
- [ ] Wire to secure storage (keyring + encrypted file fallback)
- [ ] Add auth error detection in downloads (401/403)

### Model Management
- [ ] Implement pull_model with progress tracking
- [ ] Implement delete_model with cache cleanup
- [ ] Add loaded_models tracking
- [ ] Add select_model (active model switching)

## TIER 2 - Advanced Features (Pending)

### RAG Features
- [ ] semantic_search_query
- [ ] calculate_similarity
- [ ] evaluate_embeddings
- [ ] cluster_documents
- [ ] recommend_content

### Model Discovery
- [ ] get_embedding_models
- [ ] get_recipes
- [ ] get_registered_models

## Testing Notes

**Unit Tests:**
- 42 tests total
- All passing after TIER 0 migration
- Mock states return proper typed responses

**Integration Tests:**
- Need to add end-to-end tests with real server state
- Test actual message protocol (stdin/stdout)
- Test all routes with real backend

## Notes

- Native messaging uses JSON `null` for unit structs (e.g., `HealthRequest`)
- This is different from HTTP which can use query params
- Keep middleware simple until we decide if we need it post-cleanup

