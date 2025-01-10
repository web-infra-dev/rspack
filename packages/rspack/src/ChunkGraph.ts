import type { JsChunkGraph } from "@rspack/binding";

import { Chunk } from "./Chunk";
import { Module } from "./Module";
import { DependenciesBlock } from "./DependenciesBlock";
import { ChunkGroup } from "./ChunkGroup";
import { VolatileMap } from "./util/volatile";

export class ChunkGraph {
	#inner: JsChunkGraph;

	#chunkModulesMap = new VolatileMap<Chunk, ReadonlyArray<Module>>();
	#moduleIdMap = new VolatileMap<Module, string | null>();

	static __from_binding(binding: JsChunkGraph): ChunkGraph {
		return new ChunkGraph(binding);
	}

	private constructor(binding: JsChunkGraph) {
		this.#inner = binding;
	}

	getChunkModules(chunk: Chunk): ReadonlyArray<Module> {
		let modules = this.#chunkModulesMap.get(chunk);
		if (modules === undefined) {
			modules = this.#inner
				.getChunkModules(Chunk.__to_binding(chunk))
				.map(binding => Module.__from_binding(binding));
			this.#chunkModulesMap.set(chunk, modules);
		}
		return modules;
	}

	getChunkModulesIterable(chunk: Chunk): Iterable<Module> {
		let modules = this.#chunkModulesMap.get(chunk);
		if (modules === undefined) {
			modules = this.#inner
				.getChunkModules(Chunk.__to_binding(chunk))
				.map(binding => Module.__from_binding(binding));
			this.#chunkModulesMap.set(chunk, modules);
		}
		return modules;
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
		let moduleId = this.#moduleIdMap.get(module);
		if (moduleId === undefined) {
			moduleId = this.#inner.getModuleId(Module.__to_binding(module));
			this.#moduleIdMap.set(module, moduleId);
		}
		return moduleId;
	}

	getBlockChunkGroup(depBlock: DependenciesBlock): ChunkGroup | null {
		const binding = this.#inner.getBlockChunkGroup(
			DependenciesBlock.__to_binding(depBlock)
		);
		return binding ? ChunkGroup.__from_binding(binding) : null;
	}
}
