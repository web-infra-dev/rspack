import type { JsChunkGroup } from "@rspack/binding";

import { Chunk } from "./Chunk";
import { ChunkGroup } from "./ChunkGroup";

const ENTRYPOINT_MAPPINGS = new WeakMap<JsChunkGroup, Entrypoint>();

export class Entrypoint extends ChunkGroup {
	#inner: JsChunkGroup;

	static __from_binding(binding: JsChunkGroup): Entrypoint {
		let entrypoint = ENTRYPOINT_MAPPINGS.get(binding);
		if (entrypoint) {
			return entrypoint;
		}
		entrypoint = new Entrypoint(binding);
		ENTRYPOINT_MAPPINGS.set(binding, entrypoint);
		return entrypoint;
	}

	protected constructor(binding: JsChunkGroup) {
		super(binding);
		this.#inner = binding;
	}

	getRuntimeChunk(): Readonly<Chunk | null> {
		const chunkBinding = this.#inner.getRuntimeChunk();
		return chunkBinding ? Chunk.__from_binding(chunkBinding) : null;
	}

	getEntrypointChunk(): Readonly<Chunk | null> {
		const chunkBinding = this.#inner.getEntrypointChunk();
		return chunkBinding ? Chunk.__from_binding(chunkBinding) : null;
	}
}
