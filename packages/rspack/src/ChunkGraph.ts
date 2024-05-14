import {
	__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable,
	__chunk_graph_inner_get_chunk_entry_modules,
	__chunk_graph_inner_get_chunk_modules,
	__chunk_graph_inner_get_chunk_modules_iterable_by_source_type
} from "@rspack/binding";

import { Chunk } from "./Chunk";
import { Compilation } from "./Compilation";
import { Module } from "./Module";

export class ChunkGraph {
	constructor(private compilation: Compilation) {}

	getChunkModules(chunk: Chunk): Module[] {
		return __chunk_graph_inner_get_chunk_modules(
			chunk.__internal_inner_ukey(),
			this.compilation.__internal_getInner()
		).map(m => Module.__from_binding(m));
	}

	getChunkModulesIterable(chunk: Chunk): Iterable<Module> {
		return new Set(
			__chunk_graph_inner_get_chunk_modules(
				chunk.__internal_inner_ukey(),
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m))
		);
	}

	getChunkEntryModulesIterable(chunk: Chunk): Iterable<Module> {
		return new Set(
			__chunk_graph_inner_get_chunk_entry_modules(
				chunk.__internal_inner_ukey(),
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m))
		);
	}

	getChunkEntryDependentChunksIterable(chunk: Chunk): Iterable<Chunk> {
		return new Set(
			__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable(
				chunk.__internal_inner_ukey(),
				this.compilation.__internal_getInner()
			).map(c =>
				Chunk.__from_binding(c, this.compilation.__internal_getInner())
			)
		);
	}

	getChunkModulesIterableBySourceType(
		chunk: Chunk,
		sourceType: string
	): Iterable<Module> {
		return new Set(
			__chunk_graph_inner_get_chunk_modules_iterable_by_source_type(
				chunk.__internal_inner_ukey(),
				sourceType,
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m))
		);
	}
}
