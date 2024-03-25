import { readConfigFile } from "../helper";
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
const serializer = require("jest-serializer-path");
const normalizePaths = serializer.normalizePaths;

export interface IRspackDiagnosticProcessorOptions {
	name: string;
	root: string;
}

const CWD = process.cwd();

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

	async before(context: ITestContext) {
		process.chdir(this._diagnosticOptions.root);
	}

	async after(context: ITestContext) {
		process.chdir(CWD);
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		if (!stats) {
			throw new Error("Stats should exists");
		}
		assert(stats.hasErrors() || stats.hasWarnings());
		let output = normalizePaths(
			stats.toString({
				all: false,
				errors: true,
				warnings: true
			})
		);
		// TODO: change to stats.errorStack
		if (context.getSource().includes("module-build-failed")) {
			// Replace potential loader stack
			output = output
				.replaceAll("│", "")
				.split(/\r?\n/)
				.map((s: string) => s.trim())
				.join("");
		}

		const errorOutputPath = path.resolve(context.getSource(), `./stats.err`);
		const updateSnapshot =
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot");
		if (!fs.existsSync(errorOutputPath) || updateSnapshot) {
			fs.writeFileSync(errorOutputPath, output);
		} else {
			expect(output).toBe(fs.readFileSync(errorOutputPath, "utf-8"));
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
