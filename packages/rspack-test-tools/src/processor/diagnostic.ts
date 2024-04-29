import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import { BasicTaskProcessor } from "./basic";
import assert from "assert";
import path from "path";
import fs from "fs";
import { escapeEOL } from "../helper";
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

export interface IRspackDiagnosticProcessorOptions {
	name: string;
}

export class RspackDiagnosticProcessor extends BasicTaskProcessor<ECompilerType.Rspack> {
	constructor(protected _diagnosticOptions: IRspackDiagnosticProcessorOptions) {
		super({
			defaultOptions: RspackDiagnosticProcessor.defaultOptions,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			compilerType: ECompilerType.Rspack,
			name: _diagnosticOptions.name,
			runable: false
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
			.replaceAll("│", "")
			.split(/\r?\n/)
			.map((s: string) => s.trim())
			.join("");

		const errorOutputPath = path.resolve(context.getSource(), `./stats.err`);
		if (!fs.existsSync(errorOutputPath) || global.updateSnapshot) {
			fs.writeFileSync(errorOutputPath, escapeEOL(output));
		} else {
			const expectContent = fs.readFileSync(errorOutputPath, "utf-8");
			expect(escapeEOL(output)).toBe(escapeEOL(expectContent));
		}
	}

	static defaultOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
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
