import fs from "fs";
import path from "path";

import { parseResource } from "../helper/legacy/parseResource";
import { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { IMultiTaskProcessorOptions, MultiTaskProcessor } from "./multi";

export interface IConfigProcessorOptions<T extends ECompilerType>
	extends Omit<
		IMultiTaskProcessorOptions<T>,
		"defaultOptions" | "overrideOptions" | "findBundle"
	> {}

export class ConfigProcessor<
	T extends ECompilerType
> extends MultiTaskProcessor<T> {
	constructor(protected _configOptions: IConfigProcessorOptions<T>) {
		super({
			defaultOptions: ConfigProcessor.defaultOptions<T>,
			overrideOptions: ConfigProcessor.overrideOptions<T>,
			findBundle: ConfigProcessor.findBundle<T>,
			..._configOptions
		});
	}

	static findBundle<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		const testConfig = context.getTestConfig();

		if (typeof testConfig.findBundle === "function") {
			return testConfig.findBundle!(index, options);
		}

		const ext = path.extname(parseResource(options.output?.filename).path);
		if (
			options.output?.path &&
			fs.existsSync(path.join(options.output.path!, "bundle" + index + ext))
		) {
			return "./bundle" + index + ext;
		}
	}

	static defaultOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext
	): TCompilerOptions<T> {
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
		} as TCompilerOptions<T>;
	}
	static overrideOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}
		if (!options.output?.filename) {
			const outputModule = options.experiments?.outputModule;
			options.output ??= {};
			options.output.filename = `bundle${index}${
				outputModule ? ".mjs" : ".js"
			}`;
		}
	}
}
