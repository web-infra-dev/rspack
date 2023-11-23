import { Compiler } from "../Compiler";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";

let added = false;

export class ShareRuntimeSingletonPlugin {
	apply(compiler: Compiler) {
		if (added) return;
		added = true;
		new EntryPlugin(
			compiler.context,
			require.resolve("./initializeSharing.js"),
			{ name: undefined }
		).apply(compiler);
		compiler.hooks.done.tap(ShareRuntimeSingletonPlugin.name, () => {
			added = false;
		});
	}
}
