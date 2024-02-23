import {
	ECompilerType,
	ITestContext,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { MultiTaskProcessor } from "./multi";
import path from "path";
import fs from "fs";
import { parseResource } from "../helper/legacy/parseResource";

export interface IRspackConfigProcessorOptions<T extends ECompilerType.Rspack> {
	name: string;
	testConfig: TTestConfig<T>;
}

export class RspackConfigProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(options: IRspackConfigProcessorOptions<ECompilerType.Rspack>) {
		super({
			preOptions: RspackConfigProcessor.preOptions,
			postOptions: RspackConfigProcessor.postOptions,
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: options.testConfig.findBundle
				? (index, context, compilerOptions) =>
						options.testConfig.findBundle!(index, compilerOptions)
				: RspackConfigProcessor.findBundle,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name,
			testConfig: {
				timeout: 10000,
				...options.testConfig
			}
		});
	}

	static findBundle(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	) {
		const ext = path.extname(parseResource(options.output?.filename).path);
		if (
			options.output?.path &&
			fs.existsSync(path.join(options.output.path!, "bundle" + index + ext))
		) {
			return "./bundle" + index + ext;
		}
	}

	static preOptions(
		index: number,
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
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
				rspackFuture: {
					newTreeshaking: true
				}
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
		if (!options.output?.filename) {
			const outputModule = options.experiments?.outputModule;
			options.output ??= {};
			options.output.filename = `bundle${index}${outputModule ? ".mjs" : ".js"}`;
		}
	}
}
