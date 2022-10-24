import { getNormalizedRspackOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";
import type { StatsCompilation } from "@rspack/binding";
import util from "util";
import {
	applyRspackOptionsBaseDefaults,
	applyRspackOptionsDefaults
} from "./config/defaults";
import createConsoleLogger from "./logging/createConsoleLogger";
import nodeConsole from "./node/nodeConsole";
import { Stats } from "./stats";

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
	logger.debug("RawOptions:", userOptions);

	if (Array.isArray(options.plugins)) {
		for (const plugin of options.plugins) {
			if (typeof plugin === "function") {
				plugin.call(compiler, compiler);
			} else {
				plugin.apply(compiler);
			}
		}
	}
	applyRspackOptionsDefaults(options);
	logger.debug("NormalizedOptions:", options);
	new RspackOptionsApply().process(options, compiler);
	compiler.hooks.initialize.call();
	return compiler;
}
function rspack(options: RspackOptions, callback: Callback<Stats>): Compiler {
	let compiler = createCompiler(options);
	const doRun = async () => {
		const stats = await compiler.build();
		return new Stats(compiler.compilation, stats);
	};
	if (callback) {
		util.callbackify(doRun)(callback);
		return compiler;
	} else {
		return compiler;
	}
}

// deliberately alias rspack as webpack
export { rspack, createCompiler };
