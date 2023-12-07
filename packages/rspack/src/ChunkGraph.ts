import {
	__chunk_inner_get_chunk_entry_modules,
	__chunk_inner_get_chunk_modules
} from "@rspack/binding";
import { Chunk } from "./Chunk";
import { Compilation } from "./Compilation";
import { Module } from "./Module";

export class ChunkGraph {
	constructor(private compilation: Compilation) {}

	getChunkModulesIterable(chunk: Chunk) {
		return __chunk_inner_get_chunk_modules(
			chunk.__internal_inner_ukey(),
			this.compilation.__internal_getInner()
		).map(m => Module.__from_binding(m));
	}

	getChunkEntryModulesIterable(chunk: Chunk) {
		return __chunk_inner_get_chunk_entry_modules(
			chunk.__internal_inner_ukey(),
			this.compilation.__internal_getInner()
		).map(m => Module.__from_binding(m));
	}
}
