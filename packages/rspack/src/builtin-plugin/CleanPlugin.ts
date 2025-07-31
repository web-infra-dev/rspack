import { type BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import * as liteTapable from "@rspack/lite-tapable";
import { type Compilation, checkCompilation } from "../Compilation";

import { createBuiltinPlugin, RspackBuiltinPlugin } from "./base";

export type CleanPluginHooks = {
	keep: liteTapable.SyncBailHook<[string], boolean>;
};

const compilationHooksMap: WeakMap<Compilation, CleanPluginHooks> =
	new WeakMap();

export type CleanPluginOptions = {
	/**
	 * Simulate the removal of files instead of actually removing them.
	 * @default false
	 */
	dry?: boolean;

	/**
	 * Keep specific files/patterns during clean.
	 */
	keep?: RegExp | string | ((path: string) => boolean);
};

export class CleanPlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.CleanPlugin;
	affectedHooks = "compilation" as const;

	constructor(private options: CleanPluginOptions = {}) {
		super();
	}

	raw(): BuiltinPlugin {
		const { keep, dry = false } = this.options;

		// https://github.com/webpack/webpack/blob/42daf55d3b8c442b31769865279fd06b11d9c814/lib/CleanPlugin.js#L379C3-L386C21
		const keepFn =
			typeof keep === "function"
				? keep
				: typeof keep === "string"
					? (path: string) => path.startsWith(keep)
					: typeof keep === "object" && keep.test
						? (path: string) => keep.test(path)
						: (_: string) => false;

		return createBuiltinPlugin(this.name, {
			dry,
			keep: keepFn
		});
	}

	static getCompilationHooks(compilation: Compilation): CleanPluginHooks {
		checkCompilation(compilation);

		let hooks = compilationHooksMap.get(compilation);
		if (hooks === undefined) {
			hooks = {
				keep: new liteTapable.SyncBailHook(["ignore"])
			};
			compilationHooksMap.set(compilation, hooks);
		}
		return hooks;
	}
}
