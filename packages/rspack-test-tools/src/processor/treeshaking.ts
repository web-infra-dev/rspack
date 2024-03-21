import { BasicTaskProcessor } from "./basic";
import {
	ECompilerType,
	ITestContext,
	ITestEnv,
	TCompilerOptions
} from "../type";
import path from "path";
import fs from "fs-extra";
import { merge } from "webpack-merge";
import { RuleSetRules, rspack } from "@rspack/core";

export interface IRspackTreeShakingProcessorOptions {
	name: string;
	snapshot: string;
	type: "new" | "builtin";
}

export class RspackTreeShakingProcessor extends BasicTaskProcessor<ECompilerType.Rspack> {
	constructor(
		protected _treeShakingOptions: IRspackTreeShakingProcessorOptions
	) {
		super({
			compilerType: ECompilerType.Rspack,
			defaultOptions: RspackTreeShakingProcessor.defaultOptions,
			overrideOptions: RspackTreeShakingProcessor.overrideOptions(
				_treeShakingOptions.type
			),
			name: _treeShakingOptions.name,
			runable: false
		});
	}

	async check(env: ITestEnv, context: ITestContext) {
		const compiler = this.getCompiler(context);
		const stats = compiler.getStats();
		const c = compiler.getCompiler();
		if (!stats || !c) return;

		if (stats.hasErrors()) {
			throw new Error(
				`Failed to compile in fixture ${this._options.name}, Errors: ${stats.toJson({ errors: true, all: false }).errors}`
			);
		}
		const fileContents = Object.entries(c.compilation.assets)
			.filter(([file]) => file.endsWith(".js") && !file.includes("runtime.js"))
			.map(([file, source]) => {
				const tag = path.extname(file).slice(1) || "txt";
				return `\`\`\`${tag} title=${file}\n${source.source().toString()}\n\`\`\``;
			});
		fileContents.sort();
		const content = `---\nsource: crates/rspack_testing/src/run_fixture.rs\n---\n${fileContents.join("\n\n")}\n`;
		const updateSnapshot =
			process.argv.includes("-u") || process.argv.includes("--updateSnapshot");

		const snapshotPath = path.resolve(
			context.getSource(),
			`./snapshot/${this._treeShakingOptions.snapshot}`
		);
		if (!fs.existsSync(snapshotPath) || updateSnapshot) {
			fs.writeFileSync(snapshotPath, content, "utf-8");
			return;
		}
		const snapshotContent = fs.readFileSync(snapshotPath, "utf-8");
		expect(content).toBe(snapshotContent);
	}

	static defaultOptions(
		context: ITestContext
	): TCompilerOptions<ECompilerType.Rspack> {
		let defaultOptions: TCompilerOptions<ECompilerType.Rspack> = {
			entry: {
				main: {
					import: "./index",
					runtime: "runtime"
				}
			},
			output: {
				filename: "[name].js",
				chunkFilename: "[name].js",
				cssFilename: "[name].css",
				cssChunkFilename: "[name].css",
				sourceMapFilename: "[file].map",
				chunkLoadingGlobal: "webpackChunkwebpack",
				chunkLoading: "jsonp",
				uniqueName: "__rspack_test__",
				enabledLibraryTypes: ["system"],
				strictModuleErrorHandling: false,
				iife: true,
				module: false,
				asyncChunks: true,
				scriptType: false,
				globalObject: "self",
				importFunctionName: "import"
			},
			module: {
				rules: [
					{
						test: /\.json$/,
						type: "json"
					},
					{
						test: /\.mjs$/,
						type: "js/esm"
					},
					{
						test: /\.cjs$/,
						type: "js/dynamic"
					},
					{
						test: /\.js$/,
						type: "js/auto"
					},
					{
						test: /\.css$/,
						type: "css"
					},
					{
						test: /\.wasm$/,
						type: "webassembly/async"
					}
				]
			},
			node: {
				__dirname: "mock",
				__filename: "mock",
				global: "warn"
			},
			optimization: {
				minimize: false,
				removeAvailableModules: true,
				removeEmptyChunks: true,
				moduleIds: "named",
				chunkIds: "named",
				sideEffects: false,
				mangleExports: false,
				usedExports: false,
				concatenateModules: false,
				nodeEnv: false
			},
			devtool: false,
			context: context.getSource(),
			plugins: []
		};

		const testConfigFile = context.getSource("test.config.js");
		if (fs.existsSync(testConfigFile)) {
			defaultOptions = merge(defaultOptions, require(testConfigFile));
		} else {
			const testConfigJson = context.getSource("test.config.json");
			if (fs.existsSync(testConfigJson)) {
				const content = fs
					.readFileSync(testConfigJson, "utf-8")
					.split(`"true"`)
					.join("true")
					.split(`"false"`)
					.join("false");
				const json = JSON.parse(content);
				defaultOptions = merge(defaultOptions, json);
			}
		}

		// TODO: remove these after modify all test.config.json
		const defineOptions = (defaultOptions.builtins as any)?.define;
		if (defineOptions) {
			defaultOptions.plugins!.push(new rspack.DefinePlugin(defineOptions));
			delete (defaultOptions.builtins as any)?.define;
		}

		const rules = defaultOptions.module?.rules;
		if (rules) {
			defaultOptions.module!.rules = rules.map<typeof rules>((rule: any) => {
				if (typeof rule.test?.type === "string") {
					if (rule.test.type === "regexp") {
						rule.test = new RegExp(rule.test.matcher);
					}
				}
				if (typeof rule.sideEffect === "boolean") {
					rule.sideEffects = rule.sideEffect;
					delete rule.sideEffect;
				}
				return rule;
			}) as RuleSetRules;
		}

		return defaultOptions;
	}

	static overrideOptions(type: IRspackTreeShakingProcessorOptions["type"]) {
		return (
			context: ITestContext,
			options: TCompilerOptions<ECompilerType.Rspack>
		) => {
			options.target = options.target || ["web", "es2022"];
			if (type === "new") {
				options.optimization ??= {};
				options.optimization.providedExports = true;
				options.optimization.innerGraph = true;
				options.optimization.usedExports = true;

				options.experiments ??= {};
				options.experiments.rspackFuture ??= {};
				options.experiments.rspackFuture.newTreeshaking = true;

				options.builtins ??= {};
				options.builtins.treeShaking = false;
			}
		};
	}
}
