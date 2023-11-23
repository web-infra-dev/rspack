import { Compiler } from "../Compiler";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";

let added = false;

export class RemoteRuntimeSingletonPlugin {
	apply(compiler: Compiler) {
		if (added) return;
		added = true;
		new EntryPlugin(compiler.context, require.resolve("./remotesLoading.js"), {
			name: undefined
		}).apply(compiler);
		compiler.hooks.done.tap(RemoteRuntimeSingletonPlugin.name, () => {
			added = false;
		});
	}
}
