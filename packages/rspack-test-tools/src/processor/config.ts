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

export interface IRspackConfigProcessorOptions<
	T extends ECompilerType = ECompilerType.Rspack
> {
	name: string;
	testConfig: TTestConfig<T>;
}

export class RspackConfigProcessor<
	T extends ECompilerType = ECompilerType.Rspack
> extends MultiTaskProcessor<T> {
	constructor(options: IRspackConfigProcessorOptions<T>) {
		super({
			preOptions: RspackConfigProcessor.preOptions<T>,
			postOptions: RspackConfigProcessor.postOptions<T>,
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: options.testConfig.findBundle
				? (index, context, compilerOptions) =>
						options.testConfig.findBundle!(index, compilerOptions)
				: RspackConfigProcessor.findBundle<T>,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name,
			testConfig: options.testConfig
		});
	}

	static findBundle<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		const ext = path.extname(parseResource(options.output?.filename).path);
		if (
			options.output?.path &&
			fs.existsSync(path.join(options.output.path!, "bundle" + index + ext))
		) {
			return "./bundle" + index + ext;
		}
		return "./main.js";
	}

	static preOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext
	): TCompilerOptions<T> {
		return {
			context: context.getSource(),
			mode: "development",
			target: "node",
			output: {
				path: context.getDist()
			}
		};
	}
	static postOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
		if (!options.entry) {
			options.entry = {
				main: "./"
			};
		}
	}
}
