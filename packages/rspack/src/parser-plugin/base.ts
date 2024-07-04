import * as binding from "@rspack/binding";
import { Compiler } from "../Compiler";
import { RspackPluginInstance } from "../config";

export abstract class RspackParserPlugin implements RspackPluginInstance {
	abstract raw(compiler: Compiler): binding.JsParserPlugin | undefined;
	abstract name: binding.JsParserPluginName;

	apply(compiler: Compiler) {
		let raw = this.raw(compiler);
		if (raw) {
			compiler.__internal__registerParserPlugin(raw);
		}
	}
}
export function createJsParserPlugin<R>(
	name: binding.JsParserPluginName,
	options: R
): binding.JsParserPlugin {
	return {
		name: name,
		options: options ?? false // undefined or null will cause napi error, so false for fallback
	};
}

// Parser plugins are inherited from parent compiler by default
export function create<T extends any[], R>(
	name: binding.JsParserPluginName,
	resolve: (...args: T) => R
) {
	class Plugin extends RspackParserPlugin {
		name = name;
		_options: R;

		constructor(...args: T) {
			super();
			this._options = resolve(...args);
		}

		raw(): binding.JsParserPlugin {
			return createJsParserPlugin(name, this._options);
		}
	}

	// Make the plugin class name consistent with webpack
	// https://stackoverflow.com/a/46132163
	Object.defineProperty(Plugin, "name", { value: name });

	return Plugin;
}
