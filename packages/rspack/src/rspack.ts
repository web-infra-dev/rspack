/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import assert from "assert";
import util from "util";
import { Callback } from "tapable";

import { Compiler } from "./Compiler";
import {
	MultiCompiler,
	MultiCompilerOptions,
	MultiRspackOptions
} from "./MultiCompiler";
import MultiStats from "./MultiStats";
import { Stats } from "./Stats";
import {
	RspackOptions,
	RspackPluginFunction,
	applyRspackOptionsBaseDefaults,
	applyRspackOptionsDefaults,
	getNormalizedRspackOptions,
	rspackOptions
} from "./config";
import NodeEnvironmentPlugin from "./node/NodeEnvironmentPlugin";
import { RspackOptionsApply } from "./rspackOptionsApply";
import { asArray, isNil } from "./util";
import { validate } from "./util/validate";

function createMultiCompiler(options: MultiRspackOptions): MultiCompiler {
	const compilers = options.map(createCompiler);
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
	const options = getNormalizedRspackOptions(userOptions);
	applyRspackOptionsBaseDefaults(options);
	assert(!isNil(options.context));
	const compiler = new Compiler(options.context, options);

	new NodeEnvironmentPlugin({
		infrastructureLogging: options.infrastructureLogging
	}).apply(compiler);

	if (Array.isArray(options.plugins)) {
		for (const plugin of options.plugins) {
			if (typeof plugin === "function") {
				(plugin as RspackPluginFunction).call(compiler, compiler);
			} else if (plugin) {
				plugin.apply(compiler);
			}
		}
	}
	applyRspackOptionsDefaults(compiler.options);

	compiler.hooks.environment.call();
	compiler.hooks.afterEnvironment.call();
	new RspackOptionsApply().process(compiler.options, compiler);
	compiler.hooks.initialize.call();
	return compiler;
}

function isMultiRspackOptions(o: unknown): o is MultiRspackOptions {
	return Array.isArray(o);
}

function rspack(options: MultiRspackOptions): MultiCompiler;
function rspack(options: RspackOptions): Compiler;
function rspack(
	options: MultiRspackOptions | RspackOptions
): MultiCompiler | Compiler;
function rspack(
	options: MultiRspackOptions,
	callback?: Callback<Error, MultiStats>
): null | MultiCompiler;
function rspack(
	options: RspackOptions,
	callback?: Callback<Error, Stats>
): null | Compiler;
function rspack(
	options: MultiRspackOptions | RspackOptions,
	callback?: Callback<Error, MultiStats | Stats>
): null | MultiCompiler | Compiler;
function rspack(
	options: MultiRspackOptions | RspackOptions,
	callback?: Callback<Error, MultiStats> | Callback<Error, Stats>
) {
	try {
		for (let o of asArray(options)) {
			validate(o, rspackOptions);
		}
	} catch (e) {
		if (e instanceof Error && callback) {
			callback(e);
			return null;
		}
		throw e;
	}
	const create = () => {
		if (isMultiRspackOptions(options)) {
			const compiler = createMultiCompiler(options);
			const watch = options.some(options => options.watch);
			const watchOptions = options.map(options => options.watchOptions || {});
			return { compiler, watch, watchOptions };
		}
		const compiler = createCompiler(options);
		const watch = options.watch;
		const watchOptions = options.watchOptions || {};
		return { compiler, watch, watchOptions };
	};

	if (callback) {
		try {
			const { compiler, watch, watchOptions } = create();
			if (watch) {
				compiler.watch(watchOptions as any, callback as any);
			} else {
				compiler.run((err, stats) => {
					compiler.close(() => {
						callback(err, stats as any);
					});
				});
			}
			return compiler;
		} catch (err: any) {
			process.nextTick(() => callback(err));
			return null;
		}
	} else {
		const { compiler, watch } = create();
		if (watch) {
			util.deprecate(
				() => {},
				"A 'callback' argument needs to be provided to the 'rspack(options, callback)' function when the 'watch' option is set. There is no way to handle the 'watch' option without a callback."
			)();
		}
		return compiler;
	}
}

// deliberately alias rspack as webpack
export { createCompiler, createMultiCompiler, MultiStats, rspack, Stats };
export default rspack;
