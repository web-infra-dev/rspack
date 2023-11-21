import { Compiler } from "../Compiler";
import { BuiltinPluginName, create } from "../builtin-plugin/base";

const MFScopeRuntimePlugin = create(
	BuiltinPluginName.MFScopeRuntimePlugin,
	() => undefined
);

let addded = false;

export class MFScopeRuntimeSingletonPlugin {
	apply(compiler: Compiler) {
		if (addded) return;
		addded = true;
		new MFScopeRuntimePlugin().apply(compiler);
	}
}
