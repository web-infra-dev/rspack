import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStats
} from "../type";
import path from "path";
import { rspack } from "@rspack/core";
import {
	IRspackHotProcessorOptions,
	RspackHotProcessor,
	TUpdateOptions
} from "./hot";

export interface IRspackHotStepProcessorOptions
	extends IRspackHotProcessorOptions {}

export class RspackHotStepProcessor extends RspackHotProcessor {
	private hashes: string[] = [];

	constructor(protected _hotOptions: IRspackHotProcessorOptions) {
		super(_hotOptions);
	}

	async run(env: ITestEnv, context: ITestContext) {
		context.setValue(
			this._options.name,
			"hotUpdateStepChecker",
			(
				hotUpdateContext: TUpdateOptions,
				stats: TCompilerStats<ECompilerType.Rspack>
			) => {
				const lastHash = this.hashes[this.hashes.length - 1];
				const assets = stats.toJson({ assets: true }).assets;
				console.log(assets);

				// TODO: check chunk hot update filenames

				// TODO: check chunk hot update manifest content

				// TODO: check chunk hot update filename content

				// TODO: check hot status changes

				// TODO: check propagation module path

				// TODO: check hot update result

				this.hashes.push(stats.hash!);
			}
		);
		await super.run(env, context);
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (!stats || !stats.hash) {
			expect(false);
			return;
		}
		this.hashes.push(stats.hash!);
		await super.check(env, context);
	}
}
