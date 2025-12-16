import type { RspackOptions } from "@rspack/core";
import fs from "fs-extra";
import path from "path";
import { parseResource } from "../helper/legacy/parseResource";
import { BasicCaseCreator } from "../test/creator";
import type {
	ITestContext,
	ITestEnv,
	ITestProcessor,
	TTestConfig
} from "../type";
import {
	afterExecute,
	build,
	check,
	compiler,
	configMultiCompiler,
	findMultiCompilerBundle,
	run
} from "./common";
import { createMultiCompilerRunner, getMultiCompilerRunnerKey } from "./runner";

export type TConfigCaseConfig = Omit<TTestConfig, "validate">;

export function createConfigProcessor(name: string): ITestProcessor {
	return {
		config: async (context: ITestContext) => {
			configMultiCompiler(
				context,
				name,
				["rspack.config.cjs", "rspack.config.js", "webpack.config.js"],
				defaultOptions,
				overrideOptions
			);
		},
		compiler: async (context: ITestContext) => {
			await compiler(context, name);
		},
		build: async (context: ITestContext) => {
			await build(context, name);
		},
		run: async (env: ITestEnv, context: ITestContext) => {
			await run(env, context, name, (context: ITestContext) =>
				findMultiCompilerBundle(context, name, findBundle)
			);
		},
		check: async (env: ITestEnv, context: ITestContext) => {
			await check(env, context, name);
		},
		after: async (context: ITestContext) => {
			await afterExecute(context, name);
		}
	};
}

const creator = new BasicCaseCreator({
	clean: true,
	describe: false,
	testConfig: testConfig => {
		const oldModuleScope = testConfig.moduleScope;
		testConfig.moduleScope = (ms, stats, compilerOptions) => {
			let res = ms;
			// TODO: modify runner module scope based on stats here
			if (typeof oldModuleScope === "function") {
				res = oldModuleScope(ms, stats, compilerOptions);
			}
			return res;
		};
	},
	steps: ({ name }) => [createConfigProcessor(name)],
	runner: {
		key: getMultiCompilerRunnerKey,
		runner: createMultiCompilerRunner
	},
	concurrent: true
});

export function createConfigCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

export function defaultOptions(
	index: number,
	context: ITestContext
): RspackOptions {
	return {
		context: context.getSource(),
		mode: "production",
		target: "async-node",
		devtool: false,
		cache: false,
		output: {
			path: context.getDist()
		},
		optimization: {
			minimize: false
		},
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			}
		}
	};
}

export function overrideOptions(
	index: number,
	context: ITestContext,
	options: RspackOptions
) {
	if (!options.entry) {
		options.entry = "./index.js";
	}
	if (options.amd === undefined) {
		options.amd = {};
	}
	if (!options.output?.filename) {
		const outputModule = options.experiments?.outputModule;
		options.output ??= {};
		options.output.filename = `bundle${index}${outputModule ? ".mjs" : ".js"}`;
	}

	if (options.cache === undefined) options.cache = false;
	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}

export function findBundle(
	index: number,
	context: ITestContext,
	options: RspackOptions
) {
	const testConfig = context.getTestConfig();

	if (typeof testConfig.findBundle === "function") {
		return testConfig.findBundle!(index, options);
	}

	const ext = path.extname(parseResource(options.output?.filename).path);
	const bundlePath = [];
	if (
		options.output?.path &&
		fs.existsSync(path.join(options.output.path!, `bundle${index}${ext}`))
	) {
		if (options.experiments?.css) {
			const cssOutputPath = path.join(
				options.output.path!,
				(typeof options.output?.cssFilename === "string" &&
					options.output?.cssFilename) ||
					`bundle${index}.css`
			);
			if (fs.existsSync(cssOutputPath)) {
				bundlePath.push(path.relative(options.output.path!, cssOutputPath));
			}
		}

		bundlePath.push(`./bundle${index}${ext}`);
	}
	return bundlePath;
}
