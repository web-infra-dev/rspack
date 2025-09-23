import path from "node:path";
import fs from "fs-extra";
import { rimrafSync } from "rimraf";
import {
	compareFile,
	type IFormatCodeOptions,
	type IFormatCodeReplacement
} from "../compare";
import createLazyTestEnv from "../helper/legacy/createLazyTestEnv";
import { RspackDiffConfigPlugin, WebpackDiffConfigPlugin } from "../plugin";
import { Tester } from "../test/tester";
import {
	ECompareResultType,
	ECompilerType,
	type ITestContext,
	type ITestEnv,
	type ITestProcessor,
	type TCompareModules,
	type TCompilerOptions,
	type TFileCompareResult,
	type TModuleCompareResult
} from "../type";
import { build, check, compiler, config, getCompiler } from "./common";

export type TDiffCaseConfig = IDiffProcessorOptions;

const DEFAULT_CASE_CONFIG: Partial<IDiffProcessorOptions> = {
	webpackPath: require.resolve("webpack"),
	rspackPath: require.resolve("@rspack/core"),
	files: ["bundle.js"],
	bootstrap: true,
	detail: true,
	errors: false
};

type TFileCompareItem = {
	modules: TModuleCompareResult[];
	runtimeModules: TModuleCompareResult[];
};

export function createDiffCase(name: string, src: string, dist: string) {
	const caseConfigFile = path.join(src, "test.config.js");
	if (!fs.existsSync(caseConfigFile)) {
		return;
	}
	const caseConfig: IDiffProcessorOptions = Object.assign(
		{},
		DEFAULT_CASE_CONFIG,
		require(caseConfigFile)
	);

	const [processor, compareMap] = createDiffProcessor(caseConfig);
	const tester = new Tester({
		name,
		src,
		dist,
		steps: [processor]
	});

	rimrafSync(dist);

	const prefix = path.basename(name);
	describe(`${prefix}:check`, () => {
		beforeAll(async () => {
			await tester.compile();
			compareMap.clear();
			await tester.check(env);
		});
		for (const file of caseConfig.files!) {
			describe(`Comparing "${file}"`, () => {
				let moduleResults: TModuleCompareResult[] = [];
				let runtimeResults: TModuleCompareResult[] = [];
				beforeAll(() => {
					const fileResult = compareMap.get(file);
					if (!fileResult) {
						throw new Error(`File ${file} has no results`);
					}
					moduleResults = fileResult.modules;
					runtimeResults = fileResult.runtimeModules;
				});
				if (caseConfig.modules) {
					checkCompareResults("modules", () => moduleResults);
				}
				if (caseConfig.runtimeModules) {
					checkCompareResults("runtime modules", () => runtimeResults);
				}
			});
		}
		const env = createLazyTestEnv(1000);
	});
}

function defaultOptions<T extends ECompilerType>(
	type: T,
	src: string,
	dist: string
) {
	return {
		entry: path.join(src, "./src/index.js"),
		context: src,
		output: {
			path: dist,
			filename: "bundle.js",
			chunkFilename: "[name].chunk.js"
		},
		plugins: [
			type === ECompilerType.Webpack && new WebpackDiffConfigPlugin(),
			type === ECompilerType.Rspack && new RspackDiffConfigPlugin()
		].filter(Boolean),
		experiments:
			type === ECompilerType.Rspack
				? {
						css: true,
						rspackFuture: {
							bundlerInfo: {
								force: false
							}
						}
					}
				: {}
	} as TCompilerOptions<T>;
}

export interface IDiffProcessorOptions extends IFormatCodeOptions {
	webpackPath: string;
	rspackPath: string;
	files?: string[];
	modules?: TCompareModules;
	runtimeModules?: TCompareModules;
	bootstrap?: boolean;
	detail?: boolean;
	errors?: boolean;
	replacements?: IFormatCodeReplacement[];
	renameModule?: (file: string) => string;
	onCompareFile?: (file: string, result: TFileCompareResult) => void;
	onCompareModules?: (file: string, results: TModuleCompareResult[]) => void;
	onCompareRuntimeModules?: (
		file: string,
		results: TModuleCompareResult[]
	) => void;
}

function createFormatOptions(options: IDiffProcessorOptions, hashes: string[]) {
	const formatOptions: IFormatCodeOptions = {
		ignoreModuleArguments: options.ignoreModuleArguments,
		ignoreModuleId: options.ignoreModuleId,
		ignorePropertyQuotationMark: options.ignorePropertyQuotationMark,
		ignoreBlockOnlyStatement: options.ignoreBlockOnlyStatement,
		ignoreIfCertainCondition: options.ignoreIfCertainCondition,
		ignoreSwcHelpersPath: options.ignoreSwcHelpersPath,
		ignoreObjectPropertySequence: options.ignoreObjectPropertySequence,
		ignoreCssFilePath: options.ignoreCssFilePath,
		replacements: options.replacements || []
	};
	for (const hash of hashes) {
		formatOptions.replacements!.push({ from: hash, to: "fullhash" });
	}
	return formatOptions;
}

function createDiffProcessor(options: IDiffProcessorOptions) {
	const fileCompareMap: Map<string, TFileCompareItem> = new Map();
	const createCompareResultHandler = (type: keyof TFileCompareItem) => {
		return (file: string, results: TModuleCompareResult[]) => {
			const fileResult = fileCompareMap.get(file) || {
				modules: [],
				runtimeModules: []
			};
			fileResult[type] = results;
			fileCompareMap.set(file, fileResult);
		};
	};

	const diffOptions = {
		webpackPath: options.webpackPath,
		rspackPath: options.rspackPath,
		files: options.files,
		modules: options.modules,
		runtimeModules: options.runtimeModules,
		renameModule: options.renameModule,
		ignoreModuleId: options.ignoreModuleId ?? true,
		ignoreModuleArguments: options.ignoreModuleArguments ?? true,
		ignorePropertyQuotationMark: options.ignorePropertyQuotationMark ?? true,
		ignoreBlockOnlyStatement: options.ignoreBlockOnlyStatement ?? true,
		ignoreIfCertainCondition: options.ignoreIfCertainCondition ?? true,
		ignoreSwcHelpersPath: options.ignoreSwcHelpersPath ?? true,
		ignoreObjectPropertySequence: options.ignoreObjectPropertySequence ?? true,
		ignoreCssFilePath: options.ignoreCssFilePath ?? true,
		onCompareModules: createCompareResultHandler("modules"),
		onCompareRuntimeModules: createCompareResultHandler("runtimeModules"),
		bootstrap: options.bootstrap ?? true,
		detail: options.detail ?? true,
		errors: options.errors ?? false,
		replacements: options.replacements
	} as IDiffProcessorOptions;

	const hashes: string[] = [];
	const webpackProcessor =
		global.updateSnapshot &&
		({
			config: async (context: ITestContext) => {
				const compiler = getCompiler<ECompilerType.Webpack>(
					context,
					ECompilerType.Webpack
				);
				let options = defaultOptions(
					ECompilerType.Webpack,
					context.getSource(),
					path.join(context.getDist(), ECompilerType.Webpack)
				);
				options = await config<ECompilerType.Webpack>(
					context,
					ECompilerType.Webpack,
					["webpack.config.js", "rspack.config.js"],
					options
				);
				compiler.setOptions(options);
			},
			compiler: async (context: ITestContext) => {
				await compiler<ECompilerType.Webpack>(context, ECompilerType.Webpack);
			},
			build: async (context: ITestContext) => {
				await build<ECompilerType.Webpack>(context, ECompilerType.Webpack);
			},
			run: async (env: ITestEnv, context: ITestContext) => {},
			check: async (env: ITestEnv, context: ITestContext) => {
				await check<ECompilerType.Webpack>(env, context, ECompilerType.Webpack);
			}
		} as ITestProcessor);

	const rspackProcessor = {
		config: async (context: ITestContext) => {
			const compiler = getCompiler(context, ECompilerType.Rspack);
			let options = defaultOptions(
				ECompilerType.Rspack,
				context.getSource(),
				path.join(context.getDist(), ECompilerType.Rspack)
			);
			options = await config(
				context,
				ECompilerType.Rspack,
				["rspack.config.js", "webpack.config.js"],
				options
			);
			compiler.setOptions(options);
		},
		compiler: async (context: ITestContext) => {
			await compiler(context, ECompilerType.Rspack);
		},
		build: async (context: ITestContext) => {
			await build(context, ECompilerType.Rspack);
		},
		run: async (env: ITestEnv, context: ITestContext) => {},
		check: async (env: ITestEnv, context: ITestContext) => {
			await check(env, context, ECompilerType.Rspack);
		}
	} as ITestProcessor;

	const processor = {
		config: async (context: ITestContext) => {
			if (webpackProcessor) {
				await webpackProcessor.config(context);
			}
			await rspackProcessor.config(context);
		},
		compiler: async (context: ITestContext) => {
			if (webpackProcessor) {
				await webpackProcessor.compiler(context);
			}
			await rspackProcessor.compiler(context);
		},
		build: async (context: ITestContext) => {
			if (webpackProcessor) {
				await webpackProcessor.build(context);
			}
			await rspackProcessor.build(context);
		},
		run: async (env: ITestEnv, context: ITestContext) => {},
		check: async (env: ITestEnv, context: ITestContext) => {
			if (webpackProcessor) {
				const webpackCompiler = context.getCompiler(ECompilerType.Webpack);
				const webpackStats = webpackCompiler.getStats();
				//TODO: handle chunk hash and content hash
				webpackStats?.hash && hashes.push(webpackStats?.hash);
				if (!diffOptions.errors) {
					env.expect(webpackStats?.hasErrors()).toBe(false);
				}
			}

			const rspackCompiler = context.getCompiler(ECompilerType.Rspack);
			const rspackStats = rspackCompiler.getStats();
			//TODO: handle chunk hash and content hash
			rspackStats?.hash && hashes.push(rspackStats?.hash);
			if (!diffOptions.errors) {
				env.expect(rspackStats?.hasErrors()).toBe(false);
			}

			const dist = context.getDist();
			const snapshot = context.getSource("__snapshot__");
			for (const file of diffOptions.files!) {
				const rspackDist = path.join(dist, ECompilerType.Rspack, file);
				const webpackDist = path.join(dist, ECompilerType.Webpack, file);
				const snapshotDist = path.join(
					snapshot,
					file.replace(/\.js$/, ".json")
				);
				const result = compareFile(rspackDist, webpackDist, {
					modules: diffOptions.modules,
					runtimeModules: diffOptions.runtimeModules,
					format: createFormatOptions(diffOptions, hashes),
					renameModule: diffOptions.renameModule,
					bootstrap: diffOptions.bootstrap,
					detail: diffOptions.detail,
					snapshot: snapshotDist
				});
				if (typeof diffOptions.onCompareFile === "function") {
					diffOptions.onCompareFile(file, result);
				}
				if (
					typeof diffOptions.onCompareModules === "function" &&
					result.modules.modules
				) {
					diffOptions.onCompareModules(file, result.modules.modules);
				}
				if (
					typeof diffOptions.onCompareRuntimeModules === "function" &&
					result.modules.runtimeModules
				) {
					diffOptions.onCompareRuntimeModules(
						file,
						result.modules.runtimeModules
					);
				}
			}
		}
	} as ITestProcessor;

	return [processor, fileCompareMap] as [
		ITestProcessor,
		Map<string, TFileCompareItem>
	];
}

function checkCompareResults(
	name: string,
	getResults: () => TModuleCompareResult[]
) {
	describe(`Comparing ${name}`, () => {
		it("should not miss any module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.Missing)
					.map(i => i.name)
			).toEqual([]);
		});
		it("should not have any rspack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlySource)
					.map(i => i.name)
			).toEqual([]);
		});
		it("should not have any webpack-only module", () => {
			expect(
				getResults()
					.filter(i => i.type === ECompareResultType.OnlyDist)
					.map(i => i.name)
			).toEqual([]);
		});
		it("all modules should be the same", () => {
			for (const result of getResults().filter(
				i => i.type === ECompareResultType.Different
			)) {
				console.log(`${result.name}:\n${result.detail}`);
			}
			expect(
				getResults().every(i => i.type === ECompareResultType.Same)
			).toEqual(true);
		});
	});
}
