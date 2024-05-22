import assert from "assert";
import fs from "fs";
import path from "path";

import { escapeEOL } from "../helper";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { BasicProcessor, IBasicProcessorOptions } from "./basic";
const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;
const rspackPath = path.resolve(__dirname, "../../../rspack");

const replacePaths = (input: string) => {
	const rspackRoot = normalizePaths(rspackPath);
	return normalizePaths(input).split(rspackRoot).join("<RSPACK_ROOT>");
};

declare var global: {
	updateSnapshot: boolean;
};

export interface IDiagnosticProcessorOptions<T extends ECompilerType>
	extends Omit<IBasicProcessorOptions<T>, "runable"> {
	snapshot: string;
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
		// TODO: change to stats.errorStack
		output = output
			.split("│")
			.join("")
			.split(/\r?\n/)
			.map((s: string) => s.trim())
			.join("");

		const errorOutputPath = path.resolve(
			context.getSource(this._diagnosticOptions.snapshot)
		);
		if (!fs.existsSync(errorOutputPath) || global.updateSnapshot) {
			fs.writeFileSync(errorOutputPath, escapeEOL(output));
		} else {
			const expectContent = fs.readFileSync(errorOutputPath, "utf-8");
			expect(escapeEOL(output)).toBe(escapeEOL(expectContent));
		}
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
			}
		};
	}
}
