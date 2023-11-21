import { Compiler } from "../Compiler";
import { EntryPlugin } from "../builtin-plugin/EntryPlugin";
import { ShareRuntimeSingletonPlugin } from "../sharing/ShareRuntimeSingletonPlugin";
import { MFScopeRuntimeSingletonPlugin } from "./MFScopeRuntimeSingletonPlugin";

let added = false;

export class RemoteRuntimeSingletonPlugin {
	apply(compiler: Compiler) {
		new MFScopeRuntimeSingletonPlugin().apply(compiler);
		new ShareRuntimeSingletonPlugin().apply(compiler);
		if (added) return;
		added = true;
		new EntryPlugin(compiler.context, require.resolve("./remotesLoading.js"), {
			name: undefined
		}).apply(compiler);
	}
}
