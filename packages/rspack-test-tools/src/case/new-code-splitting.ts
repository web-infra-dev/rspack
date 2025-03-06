import type { Experiments } from "@rspack/core";
import {
	ConfigProcessor,
	type IConfigProcessorOptions,
	type IStatsAPIProcessorOptions,
	StatsAPIProcessor
} from "../processor";
import { MultipleRunnerFactory } from "../runner";
import { BasicCaseCreator } from "../test/creator";
import { getSimpleProcessorRunner } from "../test/simple";
import {
	ECompilerType,
	type ITestContext,
	type TCompilerOptions
} from "../type";
import type { TStatsAPICaseConfig } from "./stats-api";

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
	},
	concurrent: 3
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

export function createStatsAPINewCodeSplittingCase(
	name: string,
	src: string,
	dist: string,
	testConfig: string
) {
	const caseConfig: TStatsAPICaseConfig = require(testConfig);
	const runner = getSimpleProcessorRunner(src, dist);

	it(caseConfig.description, async () => {
		await runner(
			name,
			new NewCodeSplittingStatsAPIProcessor({
				name: name,
				snapshotName: "NewCodeSplittingStatsOutput",
				compilerType: ECompilerType.Rspack,
				...caseConfig
			})
		);
	});
}

class NewCodeSplittingStatsAPIProcessor extends StatsAPIProcessor<ECompilerType.Rspack> {
	constructor(
		protected _statsAPIOptions: IStatsAPIProcessorOptions<ECompilerType.Rspack>
	) {
		super({
			..._statsAPIOptions,
			options: context => {
				const res = _statsAPIOptions.options?.(context) || {};
				res.experiments ??= {};
				res.experiments.parallelCodeSplitting ??= true;
				return res;
			}
		});
	}
}
