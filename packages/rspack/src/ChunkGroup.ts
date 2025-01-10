import type { JsChunkGroup } from "@rspack/binding";

import { Chunk } from "./Chunk";
import { Module } from "./Module";
import { VolatileValue } from "./util/volatile";

const CHUNK_GROUP_MAPPINGS = new WeakMap<JsChunkGroup, ChunkGroup>();

export class ChunkGroup {
	declare readonly chunks: ReadonlyArray<Chunk>;
	declare readonly index?: number;
	declare readonly name?: string;
	declare readonly origins: ReadonlyArray<ChunkGroupOrigin>;
	declare readonly childrenIterable: Set<ChunkGroup>;

	#inner: JsChunkGroup;

	#chunks = new VolatileValue<ReadonlyArray<Chunk>>();
	#name = new VolatileValue<string | undefined>();

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
					if (this.#chunks.has()) {
						return this.#chunks.get();
					}
					const chunks = this.#inner.chunks.map(binding =>
						Chunk.__from_binding(binding)
					);
					this.#chunks.set(chunks);
					return chunks;
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
					if (this.#name.has()) {
						return this.#name.get();
					}
					const name = this.#inner.name;
					this.#name.set(name);
					return name;
				}
			},
			origins: {
				enumerable: true,
				get: () => {
					return this.#inner.origins.map(origin => ({
						module: origin.module
							? Module.__from_binding(origin.module)
							: undefined,
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
}

interface ChunkGroupOrigin {
	module?: Module;
	request?: string;
}
