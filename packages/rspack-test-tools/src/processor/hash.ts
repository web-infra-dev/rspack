import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TCompilerStats,
	TTestConfig
} from "../type";
import { MultiTaskProcessor } from "./multi";

export interface IRspackHashProcessorOptions {
	name: string;
	testConfig: TTestConfig<ECompilerType.Rspack>;
}

const REG_ERROR_CASE = /error$/;

export class RspackHashProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(options: IRspackHashProcessorOptions) {
		super({
			preOptions: RspackHashProcessor.preOptions,
			postOptions: RspackHashProcessor.postOptions,
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: () => [],
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name,
			testConfig: options.testConfig
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		for (const processor of this.processors.keys()) {
			context.stats<ECompilerType.Rspack>((_, stats) => {
				if (!stats) {
					expect(false);
					return;
				}
				const statsJson = stats.toJson({ assets: true });
				if (REG_ERROR_CASE.test(this.options.name)) {
					expect((statsJson.errors || []).length > 0);
				} else {
					expect((statsJson.errors || []).length === 0);
				}

				if (typeof this.options.testConfig.validate === "function") {
					this.options.testConfig.validate(stats);
				} else {
					throw new Error(
						"HashTestCases should have test.config.js and a validate method"
					);
				}
			}, processor);
		}
	}

	static preOptions(
		index: number,
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		return {
			context: context.getSource(),
			output: {
				path: context.getDist()
			}
		};
	}
	static postOptions(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}
	}
}
