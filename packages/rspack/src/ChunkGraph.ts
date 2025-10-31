import { ChunkGraph } from "@rspack/binding";
import type { Chunk } from "./Chunk";
import type { Module } from "./Module";
import type { RuntimeSpec } from "./util/runtime";
import { toJsRuntimeSpec } from "./util/runtime";

Object.defineProperty(ChunkGraph.prototype, "getOrderedChunkModulesIterable", {
	enumerable: true,
	configurable: true,
	value(
		this: ChunkGraph,
		chunk: Chunk,
		compareFn: (_a: Module, _b: Module) => number
	): Iterable<Module> {
		const modules = this.getChunkModules(chunk);
		modules.sort(compareFn);
		return modules;
	}
});

Object.defineProperty(ChunkGraph.prototype, "getModuleChunksIterable", {
	enumerable: true,
	configurable: true,
	value(this: ChunkGraph, module: Module): Iterable<Chunk> {
		return this.getModuleChunks(module);
	}
});

Object.defineProperty(ChunkGraph.prototype, "getOrderedChunkModulesIterable", {
	enumerable: true,
	configurable: true,
	value(
		this: ChunkGraph,
		chunk: Chunk,
		compareFn: (a: Module, b: Module) => number
	): Iterable<Module> {
		const modules = this.getChunkModules(chunk);
		modules.sort(compareFn);
		return modules;
	}
});

Object.defineProperty(ChunkGraph.prototype, "getModuleHash", {
	enumerable: true,
	configurable: true,
	value(this: ChunkGraph, module: Module, runtime: RuntimeSpec): string | null {
		return this._getModuleHash(module, toJsRuntimeSpec(runtime));
	}
});

declare module "@rspack/binding" {
	interface ChunkGraph {
		getModuleChunksIterable(module: Module): Iterable<Chunk>;
		getOrderedChunkModulesIterable(
			chunk: Chunk,
			compareFn: (a: Module, b: Module) => number
		): Iterable<Module>;
		getModuleHash(module: Module, runtime: RuntimeSpec): string | null;
	}
}

export { ChunkGraph } from "@rspack/binding";
