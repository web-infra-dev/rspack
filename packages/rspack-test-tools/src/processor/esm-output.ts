import fs from "node:fs";
import path from "node:path";

import rspack from "@rspack/core";
import { parseResource } from "../helper/legacy/parseResource";
import type { ECompilerType, ITestContext, TCompilerOptions } from "../type";
import { type ISnapshotProcessorOptions, SnapshotProcessor } from "./snapshot";

export interface IConfigProcessorOptions<T extends ECompilerType>
	extends ISnapshotProcessorOptions<T> {}

export class EsmOutputProcessor extends SnapshotProcessor<ECompilerType.Rspack> {
	constructor(
		protected _configOptions: IConfigProcessorOptions<ECompilerType.Rspack>
	) {
		super({
			..._configOptions,
			defaultOptions: EsmOutputProcessor.defaultOptions,
			findBundle: EsmOutputProcessor.findBundle
		});
	}
	static findBundle<T extends ECompilerType>(
		context: ITestContext,
		options: TCompilerOptions<T>
	) {
		const testConfig = context.getTestConfig();

		if (typeof testConfig.findBundle === "function") {
			return testConfig.findBundle!(0, options);
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
						`bundle.css`
				);
				if (fs.existsSync(cssOutputPath)) {
					bundlePath.push(`./bundle.css`);
				}
			}

			bundlePath.push(`./main${ext}`);
		}

		return bundlePath;
	}

	static defaultOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		return {
			context: context.getSource(),
			mode: "production",
			target: "async-node",
			devtool: false,
			entry: "./index.js",
			cache: false,
			output: {
				path: context.getDist(),
				filename: "[name].mjs",
				chunkLoading: "import",
				chunkFormat: false,
				module: true
			},
			bail: true,
			optimization: {
				minimize: false,
				moduleIds: "named",
				chunkIds: "named",
				runtimeChunk: "single",
				removeEmptyChunks: false,
				concatenateModules: false,
				splitChunks: false
			},
			plugins: [new rspack.experiments.EsmLibraryPlugin()],
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
}
