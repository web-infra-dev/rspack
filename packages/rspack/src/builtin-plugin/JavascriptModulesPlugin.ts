import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";

import * as liteTapable from "@rspack/lite-tapable";
import type { Chunk } from "../Chunk";
import { Compilation } from "../Compilation";
import type Hash from "../util/hash";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";

export type CompilationHooks = {
	chunkHash: liteTapable.SyncHook<[Chunk, Hash]>;
};

const compilationHooksMap: WeakMap<Compilation, CompilationHooks> =
	new WeakMap();

export class JavascriptModulesPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.JavascriptModulesPlugin;
	affectedHooks = "compilation" as const;

	raw(): BuiltinPlugin {
		return createBuiltinPlugin(this.name, undefined);
	}

	static getCompilationHooks(compilation: Compilation) {
		if (!(compilation instanceof Compilation)) {
			throw new TypeError(
				"The 'compilation' argument must be an instance of Compilation"
			);
		}
		let hooks = compilationHooksMap.get(compilation);
		if (hooks === undefined) {
			hooks = {
				chunkHash: new liteTapable.SyncHook(["chunk", "hash"])
			};
			compilationHooksMap.set(compilation, hooks);
		}
		return hooks;
	}
}
