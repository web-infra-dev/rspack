import { BasicCaseCreator } from "../test/creator";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITester,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { build, compiler, configMultiCompiler, getCompiler } from "./common";

const REG_ERROR_CASE = /error$/;

export type THashCaseConfig = Pick<
	TTestConfig<ECompilerType.Rspack>,
	"validate"
>;

function defaultOptions(
	index: number,
	context: ITestContext
): TCompilerOptions<ECompilerType.Rspack> {
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
			inlineConst: true,
			lazyBarrel: true
		}
	};
}

function overrideOptions(
	index: number,
	context: ITestContext,
	options: TCompilerOptions<ECompilerType.Rspack>
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

class HashCaseCreator<T extends ECompilerType> extends BasicCaseCreator<T> {
	protected describe(
		name: string,
		tester: ITester,
		testConfig: TTestConfig<T>
	) {
		it(`should print correct hash for ${name}`, async () => {
			await tester.prepare();
			await tester.compile();
			await tester.check(this.createEnv(testConfig));
			await tester.resume();
		}, 30000);
	}
}

async function check(env: ITestEnv, context: ITestContext, name: string) {
	const compiler = getCompiler(context, name);
	const stats = compiler.getStats();
	const testConfig = context.getTestConfig();
	if (!stats) {
		env.expect(false);
		return;
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
