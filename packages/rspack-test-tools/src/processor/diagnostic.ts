import assert from "assert";
import path from "path";

import { escapeEOL } from "../helper";
import { replacePaths } from "../helper/replace-paths";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { BasicProcessor, IBasicProcessorOptions } from "./basic";
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
		let output = replacePaths(
			stats.toString({
				all: false,
				errors: true,
				warnings: true
			})
		);

		if (typeof this._diagnosticOptions.format === "function") {
			output = this._diagnosticOptions.format(output);
		}

		const errorOutputPath = path.resolve(
			context.getSource(this._diagnosticOptions.snapshot)
		);
		env.expect(escapeEOL(output)).toMatchFileSnapshot(errorOutputPath);
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
