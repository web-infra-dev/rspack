import type { JsChunk } from "@rspack/binding";

import { ChunkGroup } from "./ChunkGroup";

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

		Object.defineProperty(this, "name", {
			enumerable: true,
			get: () => {
				return binding.name;
			}
		});
		Object.defineProperty(this, "id", {
			enumerable: true,
			get: () => {
				return binding.id;
			}
		});
		Object.defineProperty(this, "ids", {
			enumerable: true,
			get: () => {
				return binding.ids;
			}
		});
		Object.defineProperty(this, "idNameHints", {
			enumerable: true,
			get: () => {
				return binding.idNameHints;
			}
		});
		Object.defineProperty(this, "filenameTemplate", {
			enumerable: true,
			get: () => {
				return binding.filenameTemplate;
			}
		});
		Object.defineProperty(this, "cssFilenameTemplate", {
			enumerable: true,
			get: () => {
				return binding.cssFilenameTemplate;
			}
		});
		Object.defineProperty(this, "files", {
			enumerable: true,
			get: () => {
				return new Set(binding.files);
			}
		});
		Object.defineProperty(this, "runtime", {
			enumerable: true,
			get: () => {
				return new Set(binding.runtime);
			}
		});
		Object.defineProperty(this, "hash", {
			enumerable: true,
			get: () => {
				return binding.hash;
			}
		});
		Object.defineProperty(this, "contentHash", {
			enumerable: true,
			get: () => {
				return binding.contentHash;
			}
		});
		Object.defineProperty(this, "renderedHash", {
			enumerable: true,
			get: () => {
				return binding.renderedHash;
			}
		});
		Object.defineProperty(this, "chunkReason", {
			enumerable: true,
			get: () => {
				return binding.chunkReason;
			}
		});
		Object.defineProperty(this, "auxiliaryFiles", {
			enumerable: true,
			get: () => {
				return new Set(binding.auxiliaryFiles);
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
}
