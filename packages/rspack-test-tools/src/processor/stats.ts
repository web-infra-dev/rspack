/* eslint-disable no-control-regex */

import { Compiler, Stats } from "@rspack/core";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions,
	TTestConfig
} from "../type";
import { MultiTaskProcessor } from "./multi";
import fs from "fs";
import path from "path";
import captureStdio from "../helper/legacy/captureStdio";

export interface IRspackStatsProcessorOptions<T extends ECompilerType.Rspack> {
	name: string;
}

const REG_ERROR_CASE = /error$/;
const quoteMeta = (str: string) => {
	return str.replace(/[-[\]\\/{}()*+?.^$|]/g, "\\$&");
};

export class RspackStatsProcessor extends MultiTaskProcessor<ECompilerType.Rspack> {
	private stderr: any;
	constructor(options: IRspackStatsProcessorOptions<ECompilerType.Rspack>) {
		super({
			defaultOptions: RspackStatsProcessor.defaultOptions,
			overrideOptions: RspackStatsProcessor.overrideOptions,
			compilerType: ECompilerType.Rspack,
			configFiles: ["rspack.config.js", "webpack.config.js"],
			name: options.name,
			runable: false
		});
	}

	async before(context: ITestContext) {
		this.stderr = captureStdio(process.stderr, true);
	}
	async after(context: ITestContext) {
		this.stderr.restore();
	}

	async compiler(context: ITestContext) {
		await super.compiler(context);
		const instance = this.getCompiler(context).getCompiler()! as any;
		const compilers = instance.compilers ? instance.compilers : [instance];
		compilers.forEach((c: Compiler) => {
			const ifs = c.inputFileSystem;
			c.inputFileSystem = Object.create(ifs);
			c.inputFileSystem.readFile = function () {
				const args = Array.prototype.slice.call(arguments);
				const callback = args.pop();
				ifs.readFile.apply(
					ifs,
					args.concat([
						(err: Error, result: Buffer) => {
							if (err) return callback(err);
							if (!/\.(js|json|txt)$/.test(args[0]))
								return callback(null, result);
							callback(null, result.toString("utf-8").replace(/\r/g, ""));
						}
					])
				);
			};

			// CHANGE: The checkConstraints() function is currently not implemented in rspack
			// c.hooks.compilation.tap("StatsTestCasesTest", compilation => {
			// 	[
			// 		"optimize",
			// 		"optimizeModules",
			// 		"optimizeChunks",
			// 		"afterOptimizeTree",
			// 		"afterOptimizeAssets",
			// 		"beforeHash"
			// 	].forEach(hook => {
			// 		compilation.hooks[hook].tap("TestCasesTest", () =>
			// 			compilation.checkConstraints()
			// 		);
			// 	});
			// });
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		const c = compiler.getCompiler();
		if (!stats || !c) return;

		for (const compilation of []
			.concat((stats as any).stats || stats)
			.map((s: any) => s.compilation)) {
			compilation.logging.delete("webpack.Compilation.ModuleProfile");
		}

		if (REG_ERROR_CASE.test(this._options.name)) {
			expect(stats.hasErrors()).toBe(true);
		} else if (stats.hasErrors()) {
			throw new Error(
				stats.toString({
					all: false,
					errors: true
					// errorStack: true,
					// errorDetails: true
				})
			);
		} else {
			fs.writeFileSync(
				path.join(context.getDist(), "stats.txt"),
				stats.toString({
					preset: "verbose",
					// context: context.getSource(),
					colors: false
				}),
				"utf-8"
			);
		}
		let toStringOptions: any = {
			context: context.getSource(),
			colors: false
		};
		let hasColorSetting = false;
		if (typeof c.options.stats !== "undefined") {
			toStringOptions = c.options.stats;
			if (toStringOptions === null || typeof toStringOptions !== "object")
				toStringOptions = { preset: toStringOptions };
			if (!toStringOptions.context)
				toStringOptions.context = context.getSource();
			hasColorSetting = typeof toStringOptions.colors !== "undefined";
		}

		if (Array.isArray(c.options) && !toStringOptions.children) {
			toStringOptions.children = c.options.map(o => o.stats);
		}

		// mock timestamps
		for (const { compilation: s } of [].concat(
			(stats as any).stats || stats
		) as Stats[]) {
			expect(s.startTime).toBeGreaterThan(0);
			expect(s.endTime).toBeGreaterThan(0);
			s.endTime = new Date("04/20/1970, 12:42:42 PM").getTime();
			s.startTime = s.endTime - 1234;
		}

		let actual = stats.toString(toStringOptions);
		expect(typeof actual).toBe("string");
		if (!hasColorSetting) {
			actual = this.stderr.toString() + actual;
			actual = actual
				.replace(/\u001b\[[0-9;]*m/g, "")
				// CHANGE: The time unit display in Rspack is second
				.replace(/[.0-9]+(\s?s)/g, "X$1");
		} else {
			actual = this.stderr.toStringRaw() + actual;
			// eslint-disable-no-control-regex
			actual = actual
				.replace(/\u001b\[1m\u001b\[([0-9;]*)m/g, "<CLR=$1,BOLD>")
				.replace(/\u001b\[1m/g, "<CLR=BOLD>")
				.replace(/\u001b\[39m\u001b\[22m/g, "</CLR>")
				.replace(/\u001b\[([0-9;]*)m/g, "<CLR=$1>")
				// CHANGE: The time unit display in Rspack is second
				.replace(/[.0-9]+(<\/CLR>)?(\s?s)/g, "X$1$2");
		}
		// cspell:ignore Xdir
		const testPath = context.getSource();
		actual = actual
			.replace(/\r\n?/g, "\n")
			// CHANGE: Remove potential line break and "|" caused by long text
			.replace(/((ERROR|WARNING)([\s\S](?!╭|├))*?)(\n  │ )/g, "$1")
			// CHANGE: Update the regular expression to replace the 'Rspack' version string
			.replace(/Rspack [^ )]+(\)?) compiled/g, "Rspack x.x.x$1 compiled")
			.replace(
				new RegExp(quoteMeta(testPath), "g"),
				"Xdir/" + this._options.name
			)
			.replace(/(\w)\\(\w)/g, "$1/$2")
			.replace(/, additional resolving: X ms/g, "")
			.replace(/Unexpected identifier '.+?'/g, "Unexpected identifier");
		expect(actual).toMatchSnapshot();
		const testConfig = context.getTestConfig();
		if (typeof testConfig?.validate === "function") {
			testConfig.validate(stats, this.stderr.toString());
		}
	}

	static defaultOptions(
		index: number,
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		if (fs.existsSync(path.join(context.getSource(), "webpack.config.js"))) {
			return {};
		}
		return {
			context: context.getSource(),
			mode: "development",
			entry: "./index.js",
			output: {
				filename: "bundle.js",
				path: context.getDist()
			},
			optimization: {
				minimize: false
			}
		};
	}
	static overrideOptions(
		index: number,
		context: ITestContext,
		options: TCompilerOptions<ECompilerType.Rspack>
	): void {
		if (!options.context) options.context = context.getSource();
		if (!options.output) options.output = options.output || {};
		if (!options.output.path) options.output.path = context.getDist();
		if (!options.plugins) options.plugins = [];
		if (!options.optimization) options.optimization = {};
		if (options.optimization.minimize === undefined) {
			options.optimization.minimize = false;
		}
	}
}
