import path from "node:path";
import { stripVTControlCharacters as stripAnsi } from "node:util";
import type { RspackOptions } from "@rspack/core";
import { diff as jestDiff } from "jest-diff";
import { TestContext } from "../test/context";
import type { ITestContext, ITestEnv, ITestProcessor } from "../type";

const CURRENT_CWD = process.cwd();

export function createDefaultsCase(name: string, src: string) {
	const caseConfig = require(src) as TDefaultsCaseConfig;
	it(`should generate the correct defaults from ${caseConfig.description}`, async () => {
		await run(name, {
			config: async (context: ITestContext) => {
				const compiler = context.getCompiler();
				compiler.setOptions(options(context, caseConfig.options));
			},
			compiler: async (context: ITestContext) => {
				const compiler = context.getCompiler();
				compiler.createCompiler();
			},
			build: async (context: ITestContext) => {
				// no need to build, just check the snapshot of compiler options
			},
			run: async (env: ITestEnv, context: ITestContext) => {
				// no need to run, just check the snapshot of compiler options
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check(env, context, name, caseConfig);
			}
		});
	});
}

export function getRspackDefaultConfig(
	cwd: string,
	config: RspackOptions
): RspackOptions {
	process.chdir(cwd);
	const { applyWebpackOptionsDefaults, getNormalizedWebpackOptions } =
		require("@rspack/core").config;
	const normalizedConfig = getNormalizedWebpackOptions(config);
	applyWebpackOptionsDefaults(normalizedConfig);
	// make snapshot stable
	(normalizedConfig as any).experiments.rspackFuture.bundlerInfo.version =
		"$version$";
	process.chdir(CURRENT_CWD);
	return normalizedConfig;
}

export type TDefaultsCaseConfig = {
	options?: (context: ITestContext) => RspackOptions;
	cwd?: string;
	diff: (
		diff: Assertion<RspackTestDiff>,
		defaults: Assertion<RspackOptions>
	) => Promise<void>;
	description: string;
};

const srcDir = path.resolve(__dirname, "../../tests/fixtures");
const distDir = path.resolve(__dirname, "../../tests/js/defaults");

function options(
	context: ITestContext,
	custom?: (context: ITestContext) => RspackOptions
) {
	let res: RspackOptions;
	if (typeof custom === "function") {
		res = custom(context);
	} else {
		res = {};
	}
	if (!("mode" in res)) {
		res.mode = "none";
	}
	return res;
}

class RspackTestDiff {
	constructor(public value: string) {}
}

async function check(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	options: {
		cwd?: string;
		diff: (
			diff: Assertion<RspackTestDiff>,
			defaults: Assertion<RspackOptions>
		) => Promise<void>;
	}
) {
	const compiler = context.getCompiler();
	const config = getRspackDefaultConfig(
		options.cwd || CURRENT_CWD,
		compiler.getOptions()
	);
	const defaultConfig = getRspackDefaultConfig(options.cwd || CURRENT_CWD, {
		mode: "none"
	});
	const diff = stripAnsi(
		jestDiff(defaultConfig, config, { expand: false, contextLines: 0 })!
	);
	await options.diff(
		env.expect(new RspackTestDiff(diff)),
		env.expect(defaultConfig)
	);
}

async function run(name: string, processor: ITestProcessor) {
	const context = new TestContext({
		name: name,
		src: srcDir,
		dist: distDir
	});
	try {
		await processor.before?.(context);
		await processor.config?.(context);
	} catch (e: unknown) {
		context.emitError(e as Error);
	} finally {
		await processor.check?.(
			{ expect, it, beforeEach, afterEach, rstest },
			context
		);
		await processor.after?.(context);
	}
}
