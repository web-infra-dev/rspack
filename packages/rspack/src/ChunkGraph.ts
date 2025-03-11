import type { JsChunkGraph } from "@rspack/binding";
import type { RuntimeSpec } from "./util/runtime";

import { Chunk } from "./Chunk";
import { ChunkGroup } from "./ChunkGroup";
import { DependenciesBlock } from "./DependenciesBlock";
import type { Module } from "./Module";
import { toJsRuntimeSpec } from "./util/runtime";

export class ChunkGraph {
	#inner: JsChunkGraph;

	static __from_binding(binding: JsChunkGraph): ChunkGraph {
		return new ChunkGraph(binding);
	}

	constructor(binding: JsChunkGraph) {
		this.#inner = binding;
	}

	getChunkModules(chunk: Chunk): ReadonlyArray<Module> {
		return this.#inner.getChunkModules(Chunk.__to_binding(chunk));
	}

	getChunkModulesIterable(chunk: Chunk): Iterable<Module> {
		return this.#inner.getChunkModules(Chunk.__to_binding(chunk));
	}

	getChunkEntryModulesIterable(chunk: Chunk): Iterable<Module> {
		return this.#inner.getChunkEntryModules(Chunk.__to_binding(chunk));
	}

	getNumberOfEntryModules(chunk: Chunk): number {
		return this.#inner.getNumberOfEntryModules(Chunk.__to_binding(chunk));
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
		return this.#inner.getChunkModulesIterableBySourceType(
			Chunk.__to_binding(chunk),
			sourceType
		);
	}

	getModuleChunks(module: Module): Chunk[] {
		return this.#inner
			.getModuleChunks(module)
			.map(binding => Chunk.__from_binding(binding));
	}

	getModuleChunksIterable(module: Module): Iterable<Chunk> {
		return this.#inner
			.getModuleChunks(module)
			.map(binding => Chunk.__from_binding(binding));
	}

	getModuleId(module: Module): string | null {
		return this.#inner.getModuleId(module);
	}

	getModuleHash(module: Module, runtime: RuntimeSpec): string | null {
		return this.#inner.getModuleHash(module, toJsRuntimeSpec(runtime));
	}

	getBlockChunkGroup(depBlock: DependenciesBlock): ChunkGroup | null {
		const binding = this.#inner.getBlockChunkGroup(
			DependenciesBlock.__to_binding(depBlock)
		);
		return binding ? ChunkGroup.__from_binding(binding) : null;
	}
}
