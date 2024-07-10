import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";

import * as liteTapable from "@rspack/lite-tapable";
import { Compilation } from "../Compilation";
import { RspackBuiltinPlugin, createBuiltinPlugin } from "./base";
import Hash = require("../util/hash");
import type { Chunk } from "../Chunk";

export type CompilationHooks = {
	chunkHash: liteTapable.SyncHook<[Chunk, Hash]>;
};

const compilationHooksMap: WeakMap<Compilation, CompilationHooks> =
	new WeakMap();

export class JavascriptModulesPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.JavascriptModulesPlugin;
	affectedHooks = "compilation" as const;

	constructor() {
		super();
	}

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
