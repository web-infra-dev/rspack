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
}

const REG_ERROR_CASE = /error$/;

export class RspackHashProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(options: IRspackHashProcessorOptions) {
		super({
			defaultOptions: RspackHashProcessor.defaultOptions,
			overrideOptions: RspackHashProcessor.overrideOptions,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name,
			runable: false
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const testConfig = context.getTestConfig();
		const stats = compiler.getStats();
		if (!stats) {
			expect(false);
			return;
		}
		const statsJson = stats.toJson({ assets: true });
		if (REG_ERROR_CASE.test(this._options.name)) {
			expect((statsJson.errors || []).length > 0);
		} else {
			expect((statsJson.errors || []).length === 0);
		}

		if (typeof testConfig.validate === "function") {
			testConfig.validate(stats);
		} else {
			throw new Error(
				"HashTestCases should have test.config.js and a validate method"
			);
		}
	}

	static defaultOptions(
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
	static overrideOptions(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}
	}
}
