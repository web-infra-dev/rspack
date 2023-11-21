import { Compiler } from "../Compiler";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";
import { BuiltinPluginName, create } from "../builtin-plugin/base";
import { MFScopeRuntimeSingletonPlugin } from "../container/MFScopeRuntimeSingletonPlugin";

const ShareRuntimePlugin = create(
	BuiltinPluginName.ShareRuntimePlugin,
	() => undefined
);

let added = false;

export class ShareRuntimeSingletonPlugin {
	apply(compiler: Compiler) {
		new MFScopeRuntimeSingletonPlugin().apply(compiler);
		if (added) return;
		added = true;
		new ShareRuntimePlugin().apply(compiler);
		new EntryPlugin(
			compiler.context,
			require.resolve("./initializeSharing.js"),
			{ name: undefined }
		).apply(compiler);
	}
}
