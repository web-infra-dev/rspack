import { BuiltinPlugin, BuiltinPluginName } from "@rspack/binding";
import {
	RspackBuiltinPlugin,
	createBuiltinPlugin
} from "../builtin-plugin/base";
import { Compiler } from "../Compiler";

const compilerSet = new WeakSet<Compiler>();

function isSingleton(compiler: Compiler) {
	return compilerSet.has(compiler);
}

function setSingleton(compiler: Compiler) {
	compilerSet.add(compiler);
}

export class ShareRuntimePlugin extends RspackBuiltinPlugin {
	name = BuiltinPluginName.ShareRuntimePlugin;

	constructor(private enhanced = false) {
		super();
	}

	raw(compiler: Compiler): BuiltinPlugin | null {
		if (isSingleton(compiler)) return null;
		setSingleton(compiler);
		return createBuiltinPlugin(this.name, this.enhanced);
	}
}
