import assert from "node:assert";
import path from "node:path";

import { normalizePlaceholder } from "../helper/expect/placeholder";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { BasicProcessor, type IBasicProcessorOptions } from "./basic";
export interface IDiagnosticProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	snapshot: string;
	format?: (output: string) => string;
}

export class DiagnosticProcessor<
	T extends ECompilerType
> extends BasicProcessor<T> {
	constructor(protected _diagnosticOptions: IDiagnosticProcessorOptions<T>) {
		super({
			defaultOptions: DiagnosticProcessor.defaultOptions<T>,
			runable: false,
			..._diagnosticOptions
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (!stats) {
			throw new Error("Stats should exists");
		}
		assert(stats.hasErrors() || stats.hasWarnings());
		let output = normalizePlaceholder(
			stats.toString({
				all: false,
				errors: true,
				warnings: true
			})
		).replaceAll("\\", "/"); // stats has some win32 paths that path-serializer can not handle

		if (typeof this._diagnosticOptions.format === "function") {
			output = this._diagnosticOptions.format(output);
		}

		const errorOutputPath = path.resolve(
			context.getSource(this._diagnosticOptions.snapshot)
		);
		env.expect(output).toMatchFileSnapshot(errorOutputPath);
	}

	static defaultOptions<T extends ECompilerType>(
		context: ITestContext
	): TCompilerOptions<T> {
		return {
			target: "node",
			context: context.getSource(),
			entry: {
				main: "./"
			},
			mode: "development",
			devServer: {
				hot: false
			},
			infrastructureLogging: {
				debug: false
			},
			output: {
				path: context.getDist()
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
}
