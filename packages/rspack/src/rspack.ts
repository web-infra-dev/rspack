import {
	getNormalizedRspackOptions,
	RspackOptions,
	applyRspackOptionsBaseDefaults,
	applyRspackOptionsDefaults,
	RspackPluginFunction
} from "./config";
import { Compiler } from "./compiler";
import { Stats } from "./stats";
import util from "util";

import { RspackOptionsApply } from "./rspackOptionsApply";
import NodeEnvironmentPlugin from "./node/NodeEnvironmentPlugin";
import {
	MultiCompiler,
	MultiCompilerOptions,
	MultiRspackOptions
} from "./multiCompiler";
import { Callback } from "tapable";
import MultiStats from "./multiStats";
import assert from "assert";
import { asArray, isNil } from "./util";
import rspackOptionsCheck from "./config/schema.check.js";

function createMultiCompiler(options: MultiRspackOptions): MultiCompiler {
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
	const options = getNormalizedRspackOptions(userOptions);
	applyRspackOptionsBaseDefaults(options);
	assert(!isNil(options.context));
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
				(plugin as RspackPluginFunction).call(compiler, compiler);
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
	compiler.hooks.environment.call();
	compiler.hooks.afterEnvironment.call();
	new RspackOptionsApply().process(compiler.options, compiler);
	compiler.hooks.initialize.call();
	return compiler;
}

function rspack(
	options: MultiCompilerOptions,
	callback?: Callback<Error, MultiStats>
): MultiCompiler;
function rspack(
	options: RspackOptions,
	callback?: Callback<Error, Stats>
): Compiler;
function rspack(options: any, callback?: Callback<Error, any>) {
	if (!asArray(options).every(i => rspackOptionsCheck(i))) {
		const detail = (rspackOptionsCheck as any).errors
			.map((e: any) => e.message)
			.join("\n");
		const title = '** Invalidate Configuration **\n';
		throw new Error(title + detail);
	}
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
