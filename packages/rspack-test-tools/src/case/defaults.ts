import path from "node:path";
import { stripVTControlCharacters as stripAnsi } from "node:util";
import { diff as jestDiff } from "jest-diff";

import { SimpleTaskProcessor } from "../processor";
import { TestContext } from "../test/context";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITestProcessor,
	type TCompiler,
	type TCompilerOptions,
	type TCompilerStats
} from "../type";

const CURRENT_CWD = process.cwd();

export function getRspackDefaultConfig(
	cwd: string,
	config: TCompilerOptions<ECompilerType>
): TCompilerOptions<ECompilerType> {
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
	options?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>;
	cwd?: string;
	diff: (
		diff: jest.JestMatchers<RspackTestDiff>,
		defaults: jest.JestMatchers<TCompilerOptions<ECompilerType.Rspack>>
	) => Promise<void>;
	description: string;
};

const srcDir = path.resolve(__dirname, "../../tests/fixtures");
const distDir = path.resolve(__dirname, "../../tests/js/defaults");

const context = new TestContext({
	src: srcDir,
	dist: distDir
});

function options(
	context: ITestContext,
	custom?: (context: ITestContext) => TCompilerOptions<ECompilerType.Rspack>
) {
	let res: TCompilerOptions<ECompilerType.Rspack>;
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
	this: SimpleTaskProcessor<ECompilerType.Rspack>,
	env: ITestEnv,
	context: ITestContext,
	options: {
		cwd?: string;
		diff: (
			diff: jest.JestMatchers<RspackTestDiff>,
			defaults: jest.JestMatchers<TCompilerOptions<ECompilerType.Rspack>>
		) => Promise<void>;
	}
) {
	const compiler = this.getCompiler(context);
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
	try {
		await processor.before?.(context);
		await processor.config?.(context);
	} catch (e: unknown) {
		context.emitError(name, e as Error);
	} finally {
		await processor.check?.(
			{ expect, it, beforeEach, afterEach, jest },
			context
		);
		await processor.after?.(context);
	}
}

export function createDefaultsCase(name: string, src: string) {
	const caseConfig = require(src) as TDefaultsCaseConfig;
	it(`should generate the correct defaults from ${caseConfig.description}`, async () => {
		await run(
			name,
			new SimpleTaskProcessor({
				name,
				compilerType: ECompilerType.Rspack,
				options: (context: ITestContext) =>
					options(context, caseConfig.options),
				check(
					this: SimpleTaskProcessor<ECompilerType.Rspack>,
					env: ITestEnv,
					context: ITestContext,
					_compiler: TCompiler<ECompilerType.Rspack>,
					_stats: TCompilerStats<ECompilerType.Rspack>
				) {
					return check.call(this, env, context, caseConfig);
				}
			})
		);
	});
}
