import { MultiTaskProcessor } from "../processor";
import { BasicCaseCreator } from "../test/creator";
import {
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITester,
	type TCompiler,
	type TCompilerMultiStats,
	type TCompilerOptions,
	type TCompilerStats,
	type TTestConfig
} from "../type";

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

async function check(
	this: MultiTaskProcessor<ECompilerType.Rspack>,
	env: ITestEnv,
	context: ITestContext,
	compiler: TCompiler<ECompilerType.Rspack>,
	stats:
		| TCompilerStats<ECompilerType.Rspack>
		| TCompilerMultiStats<ECompilerType.Rspack>
		| null
) {
	const testConfig = context.getTestConfig();
	if (!stats) {
		env.expect(false);
		return;
	}
	if (REG_ERROR_CASE.test(this._options.name)) {
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

function createHashProcessor(name: string) {
	const processor = new MultiTaskProcessor({
		name,
		compilerType: ECompilerType.Rspack,
		configFiles: ["rspack.config.js", "webpack.config.js"],
		runable: false,
		defaultOptions,
		overrideOptions,
		check
	});
	return processor;
}

const creator = new HashCaseCreator({
	clean: true,
	describe: false,
	steps: ({ name }) => [createHashProcessor(name)]
});

export function createHashCase(name: string, src: string, dist: string) {
	creator.create(name, src, dist);
}
