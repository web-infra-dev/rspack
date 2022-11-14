import { getNormalizedRspackOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";
import {
	applyRspackOptionsBaseDefaults,
	applyRspackOptionsDefaults
} from "./config/defaults";
import createConsoleLogger from "./logging/createConsoleLogger";
import nodeConsole from "./node/nodeConsole";
import { Stats } from "./stats";
import util from "util";

import { RspackOptionsApply } from "./rspackOptionsApply";
type Callback<T> = (err: Error, t: T) => void;
function createCompiler(userOptions: RspackOptions) {
	const options = getNormalizedRspackOptions(userOptions);
	applyRspackOptionsBaseDefaults(options);
	const compiler = new Compiler(options.context, options);
	const { infrastructureLogging } = options;
	compiler.infrastructureLogger = createConsoleLogger({
		level: infrastructureLogging.level || "info",
		debug: infrastructureLogging.debug || false,
		console:
			infrastructureLogging.console ||
			nodeConsole({
				colors: infrastructureLogging.colors,
				appendOnly: infrastructureLogging.appendOnly,
				stream: infrastructureLogging.stream
			})
	});
	const logger = compiler.getInfrastructureLogger("config");
	logger.debug(
		"RawOptions:",
		util.inspect(userOptions, { colors: true, depth: null })
	);

	if (Array.isArray(options.plugins)) {
		for (const plugin of options.plugins) {
			if (typeof plugin === "function") {
				plugin.call(compiler, compiler);
			} else {
				plugin.apply(compiler);
			}
		}
	}
	applyRspackOptionsDefaults(compiler.options);
	logger.debug(
		"NormalizedOptions:",
		util.inspect(compiler.options, { colors: true, depth: null })
	);
	new RspackOptionsApply().process(compiler.options, compiler);
	compiler.hooks.initialize.call();
	return compiler;
}
function rspack(options: RspackOptions, callback?: Callback<Stats>): Compiler {
	let compiler = createCompiler(options);
	if (callback) {
		compiler.run(callback);
		return compiler;
	} else {
		return compiler;
	}
}

// deliberately alias rspack as webpack
export { rspack, createCompiler };
