// CMUSearch Service Worker - Hosts WASM search engine for instant searches
// Uses importScripts() with no-modules WASM build for SW compatibility

const CACHE_NAME = 'cmusearch-v2';
const CONFIG_PATH = './src/default_config.json';

// Assets to pre-cache
const ASSETS_TO_CACHE = [
    './',
    './index.html',
    './pkg/engine.js',
    './pkg/engine_bg.wasm',
    './pkg-sw/engine.js',
    './pkg-sw/engine_bg.wasm',
    './src/default_config.json',
    './documents/courses.json',
    './documents/roomDocuments.json'
];

// IndexedDB config for search index cache
const DB_NAME = 'cmusearch-cache';
const STORE_NAME = 'index-cache';
const DB_VERSION = 1;

// Engine state
let engineReady = false;
let engineInitializing = false;

// ========== IndexedDB Helpers ==========

async function hashConfig(configJson) {
    const encoder = new TextEncoder();
    const data = encoder.encode(configJson);
    const hashBuffer = await crypto.subtle.digest('SHA-256', data);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

function openCacheDB() {
    return new Promise((resolve, reject) => {
        const request = indexedDB.open(DB_NAME, DB_VERSION);
        request.onerror = () => reject(request.error);
        request.onsuccess = () => resolve(request.result);
        request.onupgradeneeded = (event) => {
            const db = event.target.result;
            if (!db.objectStoreNames.contains(STORE_NAME)) {
                db.createObjectStore(STORE_NAME, { keyPath: 'id' });
            }
        };
    });
}

async function getCachedData(configHash) {
    try {
        const db = await openCacheDB();
        return new Promise((resolve) => {
            const tx = db.transaction(STORE_NAME, 'readonly');
            const store = tx.objectStore(STORE_NAME);
            const request = store.get('main');
            request.onsuccess = () => {
                const result = request.result;
                if (result && result.configHash === configHash) {
                    resolve(result);
                } else {
                    resolve(null);
                }
            };
            request.onerror = () => resolve(null);
        });
    } catch (e) {
        console.warn('[SW] IndexedDB unavailable:', e);
        return null;
    }
}

async function setCachedData(configHash, indexData, sourcesData) {
    try {
        const db = await openCacheDB();
        return new Promise((resolve, reject) => {
            const tx = db.transaction(STORE_NAME, 'readwrite');
            const store = tx.objectStore(STORE_NAME);
            store.put({
                id: 'main',
                configHash,
                indexData,
                sourcesData,
                timestamp: Date.now()
            });
            tx.oncomplete = () => resolve();
            tx.onerror = () => reject(tx.error);
        });
    } catch (e) {
        console.warn('[SW] Failed to cache data:', e);
    }
}

// ========== Engine Initialization ==========

async function initializeEngine() {
    if (engineReady || engineInitializing) return;
    engineInitializing = true;

    console.log('[SW] Initializing search engine...');
    const startTime = performance.now();

    try {
        // Load the no-modules WASM build using importScripts
        importScripts('./pkg-sw/engine.js');

        // Initialize WASM (wasm_bindgen is a global after importScripts)
        await wasm_bindgen('./pkg-sw/engine_bg.wasm');
        console.log('[SW] WASM loaded');

        // Fetch config
        const configResponse = await fetch(CONFIG_PATH);
        const config = await configResponse.json();
        const configJson = JSON.stringify(config);
        const configHash = await hashConfig(configJson);

        // Check IndexedDB cache
        const cached = await getCachedData(configHash);

        if (cached) {
            console.log('[SW] Restoring from cache...');
            wasm_bindgen.init_engine_from_cache(cached.indexData, cached.sourcesData, configJson);
        } else {
            console.log('[SW] Building fresh index...');

            // Fetch documents
            const sourceIds = Object.keys(config);
            const fetchPromises = sourceIds.map(async (sourceId) => {
                const { url } = config[sourceId];
                try {
                    const response = await fetch(url);
                    if (!response.ok) return [sourceId, {}];
                    return [sourceId, await response.json()];
                } catch (e) {
                    return [sourceId, {}];
                }
            });

            const docEntries = await Promise.all(fetchPromises);
            const sources = Object.fromEntries(docEntries);
            const sourcesJson = JSON.stringify(sources);

            // Build index
            wasm_bindgen.init_engine(sourcesJson, configJson);

            // Cache for next time
            const indexData = wasm_bindgen.get_cachable_index();
            await setCachedData(configHash, indexData, sourcesJson);
            console.log('[SW] Index cached');
        }

        engineReady = true;
        const duration = (performance.now() - startTime).toFixed(0);
        console.log(`[SW] Engine ready in ${duration}ms with ${wasm_bindgen.get_doc_count()} documents`);

    } catch (err) {
        console.error('[SW] Failed to initialize engine:', err);
        engineInitializing = false;
    }
}

// ========== Service Worker Events ==========

self.addEventListener('install', (event) => {
    console.log('[SW] Installing...');
    event.waitUntil(
        caches.open(CACHE_NAME)
            .then((cache) => cache.addAll(ASSETS_TO_CACHE))
            .then(() => self.skipWaiting())
            .catch((err) => console.warn('[SW] Cache failed:', err))
    );
});

self.addEventListener('activate', (event) => {
    console.log('[SW] Activating...');
    event.waitUntil(
        caches.keys()
            .then((names) => Promise.all(
                names.filter(n => n !== CACHE_NAME).map(n => caches.delete(n))
            ))
            .then(() => self.clients.claim())
            .then(() => initializeEngine()) // Initialize engine on activation
    );
});

self.addEventListener('fetch', (event) => {
    if (!event.request.url.startsWith(self.location.origin)) return;

    event.respondWith(
        caches.match(event.request)
            .then((cached) => cached || fetch(event.request))
    );
});

// Handle messages from main thread via MessageChannel
self.addEventListener('message', async (event) => {
    const { type, query, n, id } = event.data;
    const port = event.ports[0]; // Get the transferred port for responding

    if (type === 'ping') {
        port.postMessage({ type: 'pong', ready: engineReady });
        return;
    }

    if (type === 'init') {
        await initializeEngine();
        port.postMessage({ type: 'ready', docCount: engineReady ? wasm_bindgen.get_doc_count() : 0 });
        return;
    }

    if (type === 'search') {
        if (!engineReady) {
            await initializeEngine();
        }

        try {
            const startTime = performance.now();
            const results = wasm_bindgen.search_docs(query, n || 20);
            const duration = (performance.now() - startTime).toFixed(2);

            port.postMessage({
                type: 'searchResults',
                id,
                results,
                duration
            });
        } catch (err) {
            port.postMessage({
                type: 'searchError',
                id,
                error: err.message
            });
        }
        return;
    }

    if (type === 'getDocCount') {
        port.postMessage({
            type: 'docCount',
            count: engineReady ? wasm_bindgen.get_doc_count() : 0
        });
        return;
    }
});
