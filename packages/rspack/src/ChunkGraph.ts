import {
	__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable,
	__chunk_graph_inner_get_chunk_entry_modules,
	__chunk_graph_inner_get_chunk_modules,
	__chunk_graph_inner_get_chunk_modules_iterable_by_source_type,
	__chunk_graph_inner_get_module_id,
	__chunk_graph_inner_get_module_chunks
} from "@rspack/binding";

import { Chunk } from "./Chunk";
import type { Compilation } from "./Compilation";
import { Module } from "./Module";

export class ChunkGraph {
	constructor(private compilation: Compilation) {}

	getChunkModules(chunk: Chunk): Readonly<Module[]> {
		return __chunk_graph_inner_get_chunk_modules(
			chunk.__internal__innerUkey(),
			this.compilation.__internal_getInner()
		).map(m => Module.__from_binding(m, this.compilation));
	}

	getChunkModulesIterable(chunk: Chunk): Iterable<Module> {
		return new Set(
			__chunk_graph_inner_get_chunk_modules(
				chunk.__internal__innerUkey(),
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m, this.compilation))
		);
	}

	getChunkEntryModulesIterable(chunk: Chunk): Iterable<Module> {
		return new Set(
			__chunk_graph_inner_get_chunk_entry_modules(
				chunk.__internal__innerUkey(),
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m, this.compilation))
		);
	}

	getChunkEntryDependentChunksIterable(chunk: Chunk): Iterable<Chunk> {
		return new Set(
			__chunk_graph_inner_get_chunk_entry_dependent_chunks_iterable(
				chunk.__internal__innerUkey(),
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
				chunk.__internal__innerUkey(),
				sourceType,
				this.compilation.__internal_getInner()
			).map(m => Module.__from_binding(m, this.compilation))
		);
	}

	getModuleId(module: Module): string | null {
		const binding = Module.__to_binding(module);
		return __chunk_graph_inner_get_module_id(
			binding,
			this.compilation.__internal_getInner()
		);
	}

	getModuleChunksIterable(module: Module): Iterable<Chunk> {
		const binding = Module.__to_binding(module);
		const moduleChunks = __chunk_graph_inner_get_module_chunks(
			binding,
			this.compilation.__internal_getInner()
		);
		return {
			[Symbol.iterator]: () => {
				let index = 0;
				return {
					next: () => {
						if (index < moduleChunks.length) {
							return {
								value: Chunk.__from_binding(
									moduleChunks[index++],
									this.compilation
								),
								done: false
							};
						}
						return { value: undefined, done: true };
					}
				};
			}
		};
	}
}
