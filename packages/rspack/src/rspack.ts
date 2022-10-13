import { resolveOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";
import { Stats } from "./stats";
function createCompiler(userOptions: RspackOptions) {
	const options = resolveOptions(userOptions);
	const compiler = new Compiler(options.context, options);
	// todo applyRspackOptions.apply()
	compiler.hooks.initialize.call();
	return compiler;
}
async function rspack(options: RspackOptions) {
	let compiler = createCompiler(options);
	const stats = await compiler.build();
	if (stats.errors.length > 0) {
		throw new Error(stats.errors[0].message);
	}
	return stats;
}
// deliberately alias rspack as webpack
export { rspack, createCompiler };
