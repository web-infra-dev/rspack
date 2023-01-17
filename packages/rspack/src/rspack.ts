import { getNormalizedRspackOptions, RspackOptions } from "./config";
import { Compiler } from "./compiler";
import {
	applyRspackOptionsBaseDefaults,
	applyRspackOptionsDefaults
} from "./config/defaults";
import { Stats } from "./stats";
import util from "util";

import { RspackOptionsApply } from "./rspackOptionsApply";
import NodeEnvironmentPlugin from "./node/NodeEnvironmentPlugin";
import { MultiCompiler, MultiCompilerOptions } from "./multiCompiler";
import MultiStats from "./multiStats";
type Callback<T> = (err: Error, t: T) => void;

function createMultiCompiler(options: MultiCompilerOptions): MultiCompiler {
	const compilers = options.map(option => {
		const compiler = createCompiler(option);

		/**
		 * Missing features: WebpackOptionsApply
		 * `compiler.name` should be set by WebpackOptionsApply.
		 */
		compiler.name = option.name;
		return compiler;
	});
	const compiler = new MultiCompiler(
		compilers,
		options as MultiCompilerOptions
	);
	for (const childCompiler of compilers) {
		if (childCompiler.options.dependencies) {
			compiler.setDependencies(
				childCompiler,
				childCompiler.options.dependencies
			);
		}
	}

	return compiler;
}

function createCompiler(userOptions: RspackOptions): Compiler {
	// console.log("user:", userOptions);
	const options = getNormalizedRspackOptions(userOptions, () => compiler);
	applyRspackOptionsBaseDefaults(options);
	const compiler = new Compiler(options.context, options);

	new NodeEnvironmentPlugin({
		infrastructureLogging: options.infrastructureLogging
	}).apply(compiler);

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

function rspack(
	options: MultiCompilerOptions,
	callback?: Callback<MultiStats>
): MultiCompiler;
function rspack(options: RspackOptions, callback?: Callback<Stats>): Compiler;
function rspack(options: any, callback?: Callback<any>) {
	let compiler: Compiler | MultiCompiler;
	if (Array.isArray(options)) {
		compiler = createMultiCompiler(options);
	} else {
		compiler = createCompiler(options);
	}

	if (callback) {
		compiler.run(callback);
		return compiler;
	} else {
		return compiler;
	}
}

// deliberately alias rspack as webpack
export { rspack, createCompiler, createMultiCompiler };
