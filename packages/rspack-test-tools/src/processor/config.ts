import fs from "fs";
import path from "path";

import { parseResource } from "../helper/legacy/parseResource";
import {
	ECompilerType,
	ITestContext,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { MultiTaskProcessor } from "./multi";

export interface IRspackConfigProcessorOptions<T extends ECompilerType.Rspack> {
	name: string;
	runable: boolean;
}

export class RspackConfigProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	constructor(options: IRspackConfigProcessorOptions<ECompilerType.Rspack>) {
		super({
			defaultOptions: RspackConfigProcessor.defaultOptions,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			overrideOptions: RspackConfigProcessor.overrideOptions,
			findBundle: RspackConfigProcessor.findBundle,
			compilerType: ECompilerType.Rspack,
			name: options.name,
			runable: options.runable
		});
	}

	static findBundle(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
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

	static defaultOptions(
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
	static overrideOptions(
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
			options.output.filename = `bundle${index}${
				outputModule ? ".mjs" : ".js"
			}`;
		}
	}
}
