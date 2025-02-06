import type { JsChunk } from "@rspack/binding";

import { ChunkGroup } from "./ChunkGroup";
import type { EntryOptions } from "./exports";

const CHUNK_MAPPINGS = new WeakMap<JsChunk, Chunk>();

export class Chunk {
	#inner: JsChunk;

	declare readonly name?: string;
	declare readonly id?: string;
	declare readonly ids: ReadonlyArray<string>;
	declare readonly idNameHints: ReadonlyArray<string>;
	declare readonly filenameTemplate?: string;
	declare readonly cssFilenameTemplate?: string;
	declare readonly files: ReadonlySet<string>;
	declare readonly runtime: ReadonlySet<string>;
	declare readonly hash?: string;
	declare readonly contentHash: Readonly<Record<string, string>>;
	declare readonly renderedHash?: string;
	declare readonly chunkReason?: string;
	declare readonly auxiliaryFiles: ReadonlySet<string>;

	static __from_binding(binding: JsChunk) {
		let chunk = CHUNK_MAPPINGS.get(binding);
		if (chunk) {
			return chunk;
		}
		chunk = new Chunk(binding);
		CHUNK_MAPPINGS.set(binding, chunk);
		return chunk;
	}

	static __to_binding(chunk: Chunk): JsChunk {
		return chunk.#inner;
	}

	constructor(binding: JsChunk) {
		this.#inner = binding;

		Object.defineProperties(this, {
			name: {
				enumerable: true,
				get: () => {
					return binding.name;
				}
			},
			id: {
				enumerable: true,
				get: () => {
					return binding.id;
				}
			},
			ids: {
				enumerable: true,
				get: () => {
					return binding.ids;
				}
			},
			idNameHints: {
				enumerable: true,
				get: () => {
					return binding.idNameHints;
				}
			},
			filenameTemplate: {
				enumerable: true,
				get: () => {
					return binding.filenameTemplate;
				}
			},
			cssFilenameTemplate: {
				enumerable: true,
				get: () => {
					return binding.cssFilenameTemplate;
				}
			},
			files: {
				enumerable: true,
				get: () => {
					return new Set(binding.files);
				}
			},
			runtime: {
				enumerable: true,
				get: () => {
					return new Set(binding.runtime);
				}
			},
			hash: {
				enumerable: true,
				get: () => {
					return binding.hash;
				}
			},
			contentHash: {
				enumerable: true,
				get: () => {
					return binding.contentHash;
				}
			},
			renderedHash: {
				enumerable: true,
				get: () => {
					return binding.renderedHash;
				}
			},
			chunkReason: {
				enumerable: true,
				get: () => {
					return binding.chunkReason;
				}
			},
			auxiliaryFiles: {
				enumerable: true,
				get: () => {
					return new Set(binding.auxiliaryFiles);
				}
			}
		});
	}

	isOnlyInitial(): boolean {
		return this.#inner.isOnlyInitial();
	}

	canBeInitial(): boolean {
		return this.#inner.canBeInitial();
	}

	hasRuntime(): boolean {
		return this.#inner.hasRuntime();
	}

	get groupsIterable(): ReadonlySet<ChunkGroup> {
		return new Set(
			this.#inner.groups().map(binding => ChunkGroup.__from_binding(binding))
		);
	}

	getChunkMaps(realHash: boolean) {
		const chunkHashMap: Record<string | number, string> = {};
		const chunkContentHashMap: Record<
			string | number,
			Record<string, string>
		> = {};
		const chunkNameMap: Record<string | number, string> = {};

		for (const chunk of this.getAllAsyncChunks()) {
			const id = chunk.id;
			if (!id) continue;
			const chunkHash = realHash ? chunk.hash : chunk.renderedHash;
			if (chunkHash) {
				chunkHashMap[id] = chunkHash;
			}
			for (const key of Object.keys(chunk.contentHash)) {
				if (!chunkContentHashMap[key]) {
					chunkContentHashMap[key] = {};
				}
				chunkContentHashMap[key][id] = chunk.contentHash[key];
			}
			if (chunk.name) {
				chunkNameMap[id] = chunk.name;
			}
		}

		return {
			hash: chunkHashMap,
			contentHash: chunkContentHashMap,
			name: chunkNameMap
		};
	}

	getAllAsyncChunks(): ReadonlySet<Chunk> {
		return new Set(
			this.#inner
				.getAllAsyncChunks()
				.map(binding => Chunk.__from_binding(binding))
		);
	}

	getAllInitialChunks(): ReadonlySet<Chunk> {
		return new Set(
			this.#inner
				.getAllInitialChunks()
				.map(binding => Chunk.__from_binding(binding))
		);
	}

	getAllReferencedChunks(): ReadonlySet<Chunk> {
		return new Set(
			this.#inner
				.getAllReferencedChunks()
				.map(binding => Chunk.__from_binding(binding))
		);
	}

	getEntryOptions(): Readonly<EntryOptions> | undefined {
		return this.#inner.getEntryOptions();
	}
}
