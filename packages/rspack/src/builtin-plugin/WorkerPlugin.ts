import { BuiltinPlugin } from "@rspack/binding";
import { BuiltinPluginName, RspackBuiltinPlugin } from "./base";
import {
	ChunkLoading,
	OutputModule,
	WasmLoading,
	WorkerPublicPath
} from "../config";
import { Compiler } from "../Compiler";
import { EnableChunkLoadingPlugin } from "./EnableChunkLoadingPlugin";
import { EnableWasmLoadingPlugin } from "./EnableWasmLoadingPlugin";

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
		return {
			name: this.name as any,
			options: false
		};
	}
}
