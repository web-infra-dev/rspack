import type { JsChunkGraph } from "@rspack/binding";

import { Chunk } from "./Chunk";
import { Module } from "./Module";
import { DependenciesBlock } from "./DependenciesBlock";
import { ChunkGroup } from "./ChunkGroup";

export class ChunkGraph {
	#inner: JsChunkGraph;

	static __from_binding(binding: JsChunkGraph): ChunkGraph {
		return new ChunkGraph(binding);
	}

	private constructor(binding: JsChunkGraph) {
		this.#inner = binding;
	}

	getChunkModules(chunk: Chunk): ReadonlyArray<Module> {
		return this.#inner
			.getChunkModules(Chunk.__to_binding(chunk))
			.map(binding => Module.__from_binding(binding));
	}

	getChunkModulesIterable(chunk: Chunk): Iterable<Module> {
		return this.#inner
			.getChunkModules(Chunk.__to_binding(chunk))
			.map(binding => Module.__from_binding(binding));
	}

	getChunkEntryModulesIterable(chunk: Chunk): Iterable<Module> {
		return this.#inner
			.getChunkEntryModules(Chunk.__to_binding(chunk))
			.map(binding => Module.__from_binding(binding));
	}

	getChunkEntryDependentChunksIterable(chunk: Chunk): Iterable<Chunk> {
		return this.#inner
			.getChunkEntryDependentChunksIterable(Chunk.__to_binding(chunk))
			.map(binding => Chunk.__from_binding(binding));
	}

	getChunkModulesIterableBySourceType(
		chunk: Chunk,
		sourceType: string
	): Iterable<Module> {
		return this.#inner
			.getChunkModulesIterableBySourceType(
				Chunk.__to_binding(chunk),
				sourceType
			)
			.map(binding => Module.__from_binding(binding));
	}

	getModuleChunks(module: Module): Chunk[] {
		return this.#inner
			.getModuleChunks(Module.__to_binding(module))
			.map(binding => Chunk.__from_binding(binding));
	}

	getModuleChunksIterable(module: Module): Iterable<Chunk> {
		return this.#inner
			.getModuleChunks(Module.__to_binding(module))
			.map(binding => Chunk.__from_binding(binding));
	}

	getModuleId(module: Module): string | null {
		return this.#inner.getModuleId(Module.__to_binding(module));
	}

	getBlockChunkGroup(depBlock: DependenciesBlock): ChunkGroup | null {
		const binding = this.#inner.getBlockChunkGroup(
			DependenciesBlock.__to_binding(depBlock)
		);
		return binding ? ChunkGroup.__from_binding(binding) : null;
	}
}
