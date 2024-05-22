import {
	__entrypoint_inner_get_runtime_chunk,
	type JsChunkGroup,
	type JsCompilation
} from "@rspack/binding";

import { Chunk } from "./Chunk";
import { ChunkGroup } from "./ChunkGroup";

export class Entrypoint extends ChunkGroup {
	static __from_binding(chunk: JsChunkGroup, compilation: JsCompilation) {
		return new Entrypoint(chunk, compilation);
	}

	protected constructor(inner: JsChunkGroup, compilation: JsCompilation) {
		super(inner, compilation);
	}

	getRuntimeChunk(): Chunk | null {
		const c = __entrypoint_inner_get_runtime_chunk(
			this.__internal_inner_ukey(),
			this.__internal_inner_compilation()
		);
		if (c) return Chunk.__from_binding(c, this.__internal_inner_compilation());
		return null;
	}
}
