import type { JsChunkGroup } from "@rspack/binding";

import { Chunk } from "./Chunk";
import { Module } from "./Module";

const CHUNK_GROUP_MAPPINGS = new WeakMap<JsChunkGroup, ChunkGroup>();

export class ChunkGroup {
	declare readonly chunks: ReadonlyArray<Chunk>;
	declare readonly index?: number;
	declare readonly name?: string;
	declare readonly origins: ReadonlyArray<ChunkGroupOrigin>;

	#inner: JsChunkGroup;

	static __from_binding(binding: JsChunkGroup) {
		let chunkGroup = CHUNK_GROUP_MAPPINGS.get(binding);
		if (chunkGroup) {
			return chunkGroup;
		}
		chunkGroup = new ChunkGroup(binding);
		CHUNK_GROUP_MAPPINGS.set(binding, chunkGroup);
		return chunkGroup;
	}

	protected constructor(inner: JsChunkGroup) {
		this.#inner = inner;

		Object.defineProperty(this, "chunks", {
			enumerable: true,
			get: () => {
				return this.#inner.chunks.map(binding => Chunk.__from_binding(binding));
			}
		});
		Object.defineProperty(this, "index", {
			enumerable: true,
			get: () => {
				return this.#inner.index;
			}
		});
		Object.defineProperty(this, "name", {
			enumerable: true,
			get: () => {
				return this.#inner.name;
			}
		});
		Object.defineProperty(this, "origins", {
			enumerable: true,
			get: () => {
				return this.#inner.origins.map(origin => ({
					module: origin.module
						? Module.__from_binding(origin.module)
						: undefined,
					request: origin.request
				}));
			}
		});
	}

	getFiles(): ReadonlyArray<string> {
		return this.#inner.getFiles();
	}

	getParents(): ReadonlyArray<ChunkGroup> {
		return this.#inner
			.getParents()
			.map(binding => ChunkGroup.__from_binding(binding));
	}

	isInitial(): boolean {
		return this.#inner.isInitial();
	}
}

interface ChunkGroupOrigin {
	module?: Module;
	request?: string;
}
