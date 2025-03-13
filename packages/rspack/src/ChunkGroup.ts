import type { JsChunkGroup } from "@rspack/binding";

import { Chunk } from "./Chunk";
import type { Module } from "./Module";

const CHUNK_GROUP_MAPPINGS = new WeakMap<JsChunkGroup, ChunkGroup>();

export class ChunkGroup {
	declare readonly chunks: ReadonlyArray<Chunk>;
	declare readonly index?: number;
	declare readonly name?: string;
	declare readonly origins: ReadonlyArray<ChunkGroupOrigin>;
	declare readonly childrenIterable: Set<ChunkGroup>;

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

		Object.defineProperties(this, {
			chunks: {
				enumerable: true,
				get: () => {
					return this.#inner.chunks.map(binding =>
						Chunk.__from_binding(binding)
					);
				}
			},
			index: {
				enumerable: true,
				get: () => {
					return this.#inner.index;
				}
			},
			name: {
				enumerable: true,
				get: () => {
					return this.#inner.name;
				}
			},
			origins: {
				enumerable: true,
				get: () => {
					return this.#inner.origins.map(origin => ({
						module: origin.module ? origin.module : undefined,
						request: origin.request
					}));
				}
			},
			childrenIterable: {
				enumerable: true,
				get: () => {
					return this.#inner.childrenIterable.map(child =>
						ChunkGroup.__from_binding(child)
					);
				}
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

	getModulePreOrderIndex(module: Module) {
		return this.#inner.getModulePreOrderIndex(module);
	}

	getModulePostOrderIndex(module: Module) {
		return this.#inner.getModulePostOrderIndex(module);
	}
}

interface ChunkGroupOrigin {
	module?: Module;
	request?: string;
}
