// CMUSearch Shared Worker - Hosts WASM search engine shared across tabs
// Shared Workers support ES modules and can share the engine between tabs

const CONFIG_PATH = './src/default_config.json';
const DB_NAME = 'cmusearch-cache';
const STORE_NAME = 'index-cache';
const DB_VERSION = 1;

let engineReady = false;
let engineInitializing = false;
let wasmModule = null;
const connections = new Set();

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
        console.warn('[Worker] IndexedDB unavailable:', e);
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
        console.warn('[Worker] Failed to cache data:', e);
    }
}

// ========== Engine Initialization ==========

async function initializeEngine() {
    if (engineReady) return { success: true, docCount: wasmModule.get_doc_count() };
    if (engineInitializing) {
        // Wait for initialization to complete
        while (engineInitializing && !engineReady) {
            await new Promise(r => setTimeout(r, 50));
        }
        return { success: engineReady, docCount: engineReady ? wasmModule.get_doc_count() : 0 };
    }

    engineInitializing = true;
    console.log('[Worker] Initializing search engine...');
    const startTime = performance.now();

    try {
        // Import WASM module using dynamic import (works in SharedWorker)
        wasmModule = await import('./pkg/engine.js');
        await wasmModule.default();
        console.log('[Worker] WASM loaded');

        // Fetch config
        const configResponse = await fetch(CONFIG_PATH);
        const config = await configResponse.json();
        const configJson = JSON.stringify(config);
        const configHash = await hashConfig(configJson);

        // Check IndexedDB cache
        const cached = await getCachedData(configHash);

        if (cached) {
            console.log('[Worker] Restoring from cache...');
            wasmModule.init_engine_from_cache(cached.indexData, cached.sourcesData, configJson);
        } else {
            console.log('[Worker] Building fresh index...');

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
            wasmModule.init_engine(sourcesJson, configJson);

            // Cache for next time
            const indexData = wasmModule.get_cachable_index();
            await setCachedData(configHash, indexData, sourcesJson);
            console.log('[Worker] Index cached');
        }

        engineReady = true;
        engineInitializing = false;
        const duration = (performance.now() - startTime).toFixed(0);
        const docCount = wasmModule.get_doc_count();
        console.log(`[Worker] Engine ready in ${duration}ms with ${docCount} documents`);

        return { success: true, docCount, duration };

    } catch (err) {
        console.error('[Worker] Failed to initialize engine:', err);
        engineInitializing = false;
        return { success: false, error: err.message };
    }
}

// ========== Message Handling ==========

function handleMessage(port, data) {
    const { type, query, n, id } = data;

    if (type === 'ping') {
        port.postMessage({ type: 'pong', ready: engineReady });
        return;
    }

    if (type === 'init') {
        initializeEngine().then(result => {
            port.postMessage({ type: 'ready', ...result });
        });
        return;
    }

    if (type === 'search') {
        if (!engineReady) {
            initializeEngine().then(() => {
                performSearch(port, query, n, id);
            });
        } else {
            performSearch(port, query, n, id);
        }
        return;
    }

    if (type === 'getDocCount') {
        port.postMessage({
            type: 'docCount',
            count: engineReady ? wasmModule.get_doc_count() : 0
        });
        return;
    }
}

function performSearch(port, query, n, id) {
    try {
        const startTime = performance.now();
        const results = wasmModule.search_docs(query, n || 20);
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
}

// ========== Connection Handling ==========

self.onconnect = function (e) {
    const port = e.ports[0];
    connections.add(port);

    port.onmessage = function (event) {
        handleMessage(port, event.data);
    };

    port.start();
    console.log('[Worker] New connection, total:', connections.size);
};
