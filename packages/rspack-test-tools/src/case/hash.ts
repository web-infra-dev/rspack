import type { RspackOptions } from "@rspack/core";
import {
	BasicCaseCreator,
	type IBasicCaseCreatorOptions
} from "../test/creator";
import type { ITestContext, ITestEnv, ITester, TTestConfig } from "../type";
import { build, compiler, configMultiCompiler } from "./common";

const REG_ERROR_CASE = /error$/;

export type THashCaseConfig = Pick<TTestConfig, "validate">;

class HashCaseCreator extends BasicCaseCreator {
	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig,
		options: IBasicCaseCreatorOptions
	) {
		it(`should print correct hash for ${name}`, async () => {
			await tester.prepare();
			await tester.compile();
			await tester.check(this.createEnv(testConfig, options));
			await tester.resume();
		}, 30000);
	}
}

const creator = new HashCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [
		{
			config: async (context: ITestContext) => {
				configMultiCompiler(
					context,
					name,
					["rspack.config.js", "webpack.config.js"],
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
				// no need to run, just check snapshot
			},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check(env, context, name);
			}
		}
	]
});

export function createHashCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}

function defaultOptions(index: number, context: ITestContext): RspackOptions {
	return {
		context: context.getSource(),
		output: {
			path: context.getDist()
		},
		experiments: {
			css: true,
			rspackFuture: {
				bundlerInfo: {
					force: false
				}
			},
			inlineConst: true
		}
	};
}

function overrideOptions(
	index: number,
	context: ITestContext,
	options: RspackOptions
) {
	if (!options.entry) {
		options.entry = "./index.js";
	}
	if (!global.printLogger) {
		options.infrastructureLogging = {
			level: "error"
		};
	}
}

async function check(env: ITestEnv, context: ITestContext, name: string) {
	const compiler = context.getCompiler();
	const stats = compiler.getStats();
	const testConfig = context.getTestConfig();
	if (!stats) {
		throw new Error(
			"No stats found\n" +
				context
					.getError()
					.map(e => e.stack)
					.join("\n")
		);
	}
	if (REG_ERROR_CASE.test(name)) {
		env.expect(stats.hasErrors());
	} else {
		env.expect(!stats.hasErrors());
	}

	if (typeof testConfig.validate === "function") {
		testConfig.validate(stats);
	} else {
		throw new Error(
			"HashTestCases should have test.config.js and a validate method"
		);
	}
}
