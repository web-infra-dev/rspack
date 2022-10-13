import { resolve } from "node:path";
import { resolveOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";

function createCompiler(userOptions: RspackOptions) {
	const options = resolveOptions(userOptions);
	const compiler = new Compiler(options.context, options);
	// todo applyRspackOptions.apply()
	compiler.hooks.initialize.call();
	return compiler;
}
function rspack(options: RspackOptions) {
	let compiler = createCompiler(options);
	return {
		compiler
	};
}

export { rspack };
