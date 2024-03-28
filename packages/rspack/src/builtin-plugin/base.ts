import * as binding from "@rspack/binding";
import { Compiler, RspackPluginInstance } from "..";

type AffectedHooks = keyof Compiler["hooks"];

export const HOOKS_CAN_NOT_INHERENT_FROM_PARENT = [
	"make",
	"compile",
	"emit",
	"afterEmit",
	"invalid",
	"done",
	"thisCompilation"
];

export function canInherentFromParent(affectedHooks?: AffectedHooks): boolean {
	if (typeof affectedHooks === "undefined") {
		// this arm should be removed
		return false;
	} else {
		return !HOOKS_CAN_NOT_INHERENT_FROM_PARENT.includes(affectedHooks);
	}
}

export abstract class RspackBuiltinPlugin implements RspackPluginInstance {
	abstract raw(compiler: Compiler): binding.BuiltinPlugin | undefined;
	abstract name: binding.BuiltinPluginName;

	affectedHooks?: AffectedHooks;
	apply(compiler: Compiler) {
		let raw = this.raw(compiler);
		if (raw) {
			raw.canInherentFromParent = canInherentFromParent(this.affectedHooks);
			compiler.__internal__registerBuiltinPlugin(raw);
		}
	}
}

export function createBuiltinPlugin<R>(
	name: binding.BuiltinPluginName,
	options: R
): binding.BuiltinPlugin {
	return {
		name: name as any,
		options: options ?? false // undefined or null will cause napi error, so false for fallback
	};
}

export function create<T extends any[], R>(
	name: binding.BuiltinPluginName,
	resolve: (...args: T) => R,
	// `affectedHooks` is used to inform `createChildCompile` about which builtin plugin can be reserved.
	// However, this has a drawback as it doesn't represent the actual condition but merely serves as an indicator.
	affectedHooks?: AffectedHooks
) {
	class Plugin extends RspackBuiltinPlugin {
		name = name;
		_options: R;
		affectedHooks = affectedHooks;

		constructor(...args: T) {
			super();
			this._options = resolve(...args);
		}

		raw(): binding.BuiltinPlugin {
			return createBuiltinPlugin(name, this._options);
		}
	}

	// Make the plugin class name consistent with webpack
	// https://stackoverflow.com/a/46132163
	Object.defineProperty(Plugin, "name", { value: name });

	return Plugin;
}
