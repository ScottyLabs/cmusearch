# CMUSearch Engine

WASM-based search engine with trigram indexing and BM25 ranking.

## Quick Start

```bash
# Build WASM
wasm-pack build --target web

# Serve locally
python3 -m http.server 8080

# Open http://localhost:8080/index.html
```

## Architecture

### Search Engine (Rust → WASM)
- **Trigram indexing**: Breaks text into 4-character n-grams for fuzzy matching
- **BM25 ranking**: Industry-standard relevance scoring
- **Field weighting**: Configurable per-field importance (see `src/default_config.json`)

### Caching Strategy

```
┌─────────────┐     ┌──────────────────┐     ┌───────────┐
│  IndexedDB  │ ←── │  Shared Worker   │ ←── │   Main    │
│   (cache)   │     │  (WASM engine)   │     │   Page    │
└─────────────┘     └──────────────────┘     └───────────┘
```

1. **Shared Worker** (`search-worker.js`): Hosts the WASM engine
   - Shares engine across all tabs (open 2 tabs = 1 engine)
   - Supports ES modules (unlike Service Worker)
   - Communicates via `postMessage`

2. **IndexedDB Cache**: Stores pre-built search index
   - Config hash versioning: auto-invalidates when config changes
   - First load: ~500-1000ms (build index)
   - Cached load: ~50-100ms (restore from cache)

3. **Service Worker** (`sw.js`): Caches static assets (optional)
   - WASM files, documents, HTML
   - Enables offline capability

### Why Shared Worker instead of Service Worker?

Service Workers have browser limitations:
- ❌ No ES module `import()` support
- ❌ `importScripts()` with WASM bindings is problematic

Shared Workers:
- ✅ Full ES module support via `{ type: 'module' }`
- ✅ Shared across tabs
- ✅ Stays alive while any tab is open

## Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | WASM exports: `init_engine`, `search_docs`, etc. |
| `src/build.rs` | Builds inverted index from documents |
| `src/parse.rs` | Trigram generation and query parsing |
| `src/rank.rs` | BM25 scoring algorithm |
| `src/types.rs` | Type definitions |
| `search-worker.js` | Shared Worker hosting WASM engine |
| `sw.js` | Service Worker for asset caching |
| `index.html` | Frontend UI |
| `src/default_config.json` | Document sources and field weights |

## Config Format

```json
{
  "courses": {
    "url": "./documents/courses.json",
    "weights": {
      "name": 0.2,
      "courseID": 0.6,
      "desc": 0.1,
      "department": 0.1
    }
  }
}
```

## Development

```bash
# Run tests (native only, not WASM)
cargo test

# Build with debug info
wasm-pack build --target web --dev
```
