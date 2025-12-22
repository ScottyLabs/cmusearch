declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	
	/**
	 * Get the cachable index data as JSON for IndexedDB storage
	 * Returns the expensive-to-compute index data that can be cached
	 */
	export function get_cachable_index(): string;
	
	/**
	 * Get total number of documents
	 */
	export function get_doc_count(): number;
	
	/**
	 * Greet function for testing WASM is working
	 */
	export function greet(name: string): string;
	
	/**
	 * Initialize the search engine with documents and config from JavaScript
	 * docs_json: JSON string of SourcesStore (HashMap<source_id, HashMap<doc_id, HashMap<field, value>>>)
	 * config_json: JSON string of SourcesConfig (HashMap<source_id, SourceConfig>)
	 */
	export function init_engine(docs_json: string, config_json: string): void;
	
	/**
	 * Initialize the search engine from cached index data
	 * cached_json: JSON string of CachableIndex (from get_cachable_index)
	 * docs_json: JSON string of SourcesStore (still needed for result lookups)
	 * config_json: JSON string of SourcesConfig (for field weights during search)
	 */
	export function init_engine_from_cache(cached_json: string, docs_json: string, config_json: string): void;
	
	/**
	 * Check if the engine has been initialized
	 */
	export function is_engine_ready(): boolean;
	
	/**
	 * Search all documents and return JSON results
	 */
	export function search_docs(query: string, n: number): string;
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_cachable_index: () => [number, number, number, number];
  readonly get_doc_count: () => [number, number, number];
  readonly greet: (a: number, b: number) => [number, number];
  readonly init_engine: (a: number, b: number, c: number, d: number) => [number, number];
  readonly init_engine_from_cache: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly is_engine_ready: () => number;
  readonly search_docs: (a: number, b: number, c: number) => [number, number, number, number];
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
declare function wasm_bindgen (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
