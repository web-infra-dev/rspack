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
	private root: string = process.cwd();
	constructor(options: IRspackDiagnosticProcessorOptions) {
		super({
			preOptions: RspackDiagnosticProcessor.preOptions,
			getCompiler: () => require("@rspack/core").rspack,
			getBundle: () => [],
			getCompilerOptions: context =>
				readConfigFile<ECompilerType.Rspack>(context.getSource(), [
					"rspack.config.js",
					"webpack.config.js"
				])[0],
			name: options.name,
			testConfig: {}
		});
		this.root = options.root;
	}

	async before(context: ITestContext) {
		process.chdir(this.root);
	}

	async after(context: ITestContext) {
		process.chdir(CWD);
	}

	async check(env: ITestEnv, context: ITestContext) {
		context.stats<ECompilerType.Rspack>((error, stats) => {
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
				process.argv.includes("-u") ||
				process.argv.includes("--updateSnapshot");
			if (!fs.existsSync(errorOutputPath) || updateSnapshot) {
				fs.writeFileSync(errorOutputPath, output);
			} else {
				expect(output).toBe(fs.readFileSync(errorOutputPath, "utf-8"));
			}
		}, this.options.name);
	}

	static preOptions(
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
