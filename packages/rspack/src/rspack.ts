import { getNormalizedRspackOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";
import type { StatsDescription } from "@rspack/binding";
import util from "util";
type Callback<T> = (err: Error, t: T) => void;
function createCompiler(userOptions: RspackOptions) {
	const options = getNormalizedRspackOptions(userOptions);
	const compiler = new Compiler(options.context, options);
	// todo applyRspackOptions.apply()
	compiler.hooks.initialize.call();
	return compiler;
}
function rspack(options: RspackOptions, callback: Callback<StatsDescription>): Compiler {
	let compiler = createCompiler(options);
	const doRun = async () => {
		const stats = await compiler.build();
		if (stats.errors.length > 0) {
			throw new Error(stats.errors[0].message);
		}
		return stats;
	};
	if (callback) {
		util.callbackify(doRun)(callback);
	} else {
		return compiler;
	}
}

// deliberately alias rspack as webpack
export { rspack, createCompiler };
