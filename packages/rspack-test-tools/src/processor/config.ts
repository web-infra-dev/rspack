import fs from "node:fs";
import path from "node:path";

import { parseResource } from "../helper/legacy/parseResource";
import type { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { type IMultiTaskProcessorOptions, MultiTaskProcessor } from "./multi";

export interface IConfigProcessorOptions<T extends ECompilerType>
	extends IMultiTaskProcessorOptions<T> {}

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
		const bundlePath = [];
		if (
			options.output?.path &&
			fs.existsSync(path.join(options.output.path!, `bundle${index}${ext}`))
		) {
			if (options.experiments?.css) {
				const cssOutputPath = path.join(
					options.output.path!,
					(typeof options.output?.cssFilename === "string" &&
						options.output?.cssFilename) ||
						`bundle${index}.css`
				);
				if (fs.existsSync(cssOutputPath)) {
					bundlePath.push(`./bundle${index}.css`);
				}
			}

			bundlePath.push(`./bundle${index}${ext}`);
		}

		return bundlePath;
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
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
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
		if (!global.printLogger) {
			options.infrastructureLogging = {
				level: "error"
			};
		}
	}
}
