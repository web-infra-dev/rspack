import fs from "node:fs";
import path from "node:path";

import rspack from "@rspack/core";
import { parseResource } from "../helper/legacy/parseResource";
import type { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { type IMultiTaskProcessorOptions, MultiTaskProcessor } from "./multi";

export interface IConfigProcessorOptions<T extends ECompilerType>
	extends IMultiTaskProcessorOptions<T> {}

export class EsmOutputProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(
		protected _configOptions: IConfigProcessorOptions<ECompilerType.Rspack>
	) {
		super({
			defaultOptions: EsmOutputProcessor.defaultOptions,
			overrideOptions: EsmOutputProcessor.overrideOptions,
			findBundle: EsmOutputProcessor.findBundle,
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
			fs.existsSync(path.join(options.output.path!, `main${ext}`))
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

			bundlePath.push(`./main${ext}`);
		}

		return bundlePath;
	}

	static defaultOptions(
		_index: number,
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		return {
			context: context.getSource(),
			mode: "production",
			target: "async-node",
			devtool: false,
			cache: false,
			output: {
				path: context.getDist(),
				filename: "[name].mjs",
				chunkLoading: "import",
				chunkFormat: "esm"
			},
			optimization: {
				minimize: false,
				runtimeChunk: "single",
				concatenateModules: false,
				splitChunks: false
			},
			plugins: [new rspack.experiments.RemoveDuplicateModulesPlugin()],
			experiments: {
				css: true,
				rspackFuture: {
					bundlerInfo: {
						force: false
					}
				},
				outputModule: true
			}
		};
	}
	static overrideOptions<T extends ECompilerType>(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	): void {
		if (!options.entry) {
			options.entry = "./index.js";
		}
		if (!global.printLogger) {
			options.infrastructureLogging = {
				level: "error"
			};
		}
		options.experiments ??= {};
		options.experiments.outputModule = true;
	}
}
