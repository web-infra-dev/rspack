import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";

import type { Compiler } from "../Compiler";
import type {
	ChunkLoading,
	OutputModule,
	WasmLoading,
	WorkerPublicPath
} from "../config";
import { EnableChunkLoadingPlugin } from "./EnableChunkLoadingPlugin";
import { EnableWasmLoadingPlugin } from "./EnableWasmLoadingPlugin";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export class WorkerPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.WorkerPlugin;

	constructor(
		private chunkLoading: ChunkLoading,
		private wasmLoading: WasmLoading,
		private module: OutputModule,
		private workerPublicPath: WorkerPublicPath
	) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin {
		if (this.chunkLoading) {
			new EnableChunkLoadingPlugin(this.chunkLoading).apply(compiler);
		}
		if (this.wasmLoading) {
			new EnableWasmLoadingPlugin(this.wasmLoading).apply(compiler);
		}
		return createBuiltinPlugin(this.name, undefined);
	}
}
