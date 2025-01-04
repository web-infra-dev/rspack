import { Experiments } from "@rspack/core";
import { ConfigProcessor, IConfigProcessorOptions } from "../processor";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { ECompilerType, ITestContext, TCompilerOptions } from "../type";

export function createConfigNewCodeSplittingCase(
	name: string,
	src: string,
	dist: string
) {
	configCreator.create(name, src, dist);
}

const configCreator = new BasicCaseCreator({
	clean: true,
	runner: MultipleRunnerFactory,
	describe: false,
	testConfig: testConfig => {
		const oldModuleScope = testConfig.moduleScope;
		testConfig.moduleScope = (ms, stats) => {
			let res = ms;
			// TODO: modify runner module scope based on stats here
			if (typeof oldModuleScope === "function") {
				res = oldModuleScope(ms, stats);
			}
			return res;
		};
	},
	steps: ({ name }) => {
		const processor = new NewCodeSplittingProcessor({
			name,
			runable: true,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"]
		});
		return [processor];
	}
});

class NewCodeSplittingProcessor<
	T extends ECompilerType
> extends ConfigProcessor<T> {
	constructor(protected _configOptions: IConfigProcessorOptions<T>) {
		super({
			..._configOptions,
			overrideOptions: NewCodeSplittingProcessor.overrideOptions<T>
		});
	}

	static overrideOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		ConfigProcessor.overrideOptions(index, context, options);
		options.experiments ??= {};
		(options.experiments as Experiments).parallelCodeSplitting ??= true;
	}
}
