import {
	ECompilerType,
	ITestContext,
	ITestProcessor,
	TCompilerOptions
} from "../../type";
import fs from "fs-extra";
import path from "path";
import { webpack } from "webpack";
import { rspack } from "@rspack/core";
import deepmerge from "deepmerge";
import { parseBundleModules } from "./parse-bundle-module";
import { formatCode } from "./format-code";
import { replaceRuntimeModuleName } from "./replace-runtime-module-name";
import { createModulePlaceholderPlugin } from "../../plugin/module-placeholder";

const OUTPUT_MAIN_FILE = "bundle.js";

export interface IDiffBuilderOptions {
	webpackDist: string;
	rspackDist: string;
	modules: string[] | boolean;
	runtimeModules: string[] | boolean;
	ignoreModuleId: boolean;
}

export class DiffBuilder implements ITestProcessor {
	constructor(private options: IDiffBuilderOptions) {}

	async config(context: ITestContext) {
		const source = context.getSource();
		const dist = context.getDist();

		// base configuration
		context.setOptions<ECompilerType.Rspack>(
			options =>
				this.setDefaultOptions<ECompilerType.Rspack>(
					options,
					ECompilerType.Rspack,
					source,
					path.join(dist, "rspack")
				),
			"rspack"
		);
		context.setOptions<ECompilerType.Webpack>(
			options =>
				this.setDefaultOptions<ECompilerType.Webpack>(
					options,
					ECompilerType.Webpack,
					source,
					path.join(dist, "webpack")
				),
			"webpack"
		);
		// test case configuration
		try {
			context.setOptions<ECompilerType.Rspack>(
				options =>
					deepmerge(
						options,
						this.readConfigFiles(source, ["rspack.config.js"])
					),
				"rspack"
			);
			context.setOptions<ECompilerType.Webpack>(
				options =>
					deepmerge(
						options,
						this.readConfigFiles(source, ["webpack.config.js"])
					),
				"webpack"
			);
		} catch (e) {
			context.emitError(e);
		}
		// diff configuration
		context.setOptions<ECompilerType.Rspack>(options => {
			options.output!.path = this.options.rspackDist;
			return options;
		}, "rspack");
		context.setOptions<ECompilerType.Webpack>(options => {
			options.output!.path = this.options.webpackDist;
			return options;
		}, "webpack");
	}
	async compiler(context: ITestContext) {
		context.setCompiler<ECompilerType.Rspack>(
			options => rspack({ ...options }),
			"rspack"
		);
		context.setCompiler<ECompilerType.Webpack>(
			options => webpack({ ...options }),
			"webpack"
		);
	}
	async build(context: ITestContext) {
		await context.build<ECompilerType.Rspack>(
			compiler =>
				new Promise<void>((resolve, reject) => {
					compiler.run((error, stats) => {
						if (error) return reject(error);
						if (stats) context.setStats(() => stats);
						resolve();
					});
				}),
			"rspack"
		);
		await context.build<ECompilerType.Webpack>(
			compiler =>
				new Promise<void>((resolve, reject) => {
					compiler.run((error, stats) => {
						if (error) return reject(error);
						if (stats) context.setStats(() => stats);
						resolve();
					});
				}),
			"webpack"
		);
	}
	async check(context: ITestContext) {
		const rspackDistContent = fs.readFileSync(
			path.join(this.options.rspackDist, OUTPUT_MAIN_FILE),
			"utf-8"
		);
		const webpackDistContent = replaceRuntimeModuleName(
			fs.readFileSync(
				path.join(this.options.webpackDist, OUTPUT_MAIN_FILE),
				"utf-8"
			)
		);
		const rspackModules = parseBundleModules(rspackDistContent);
		const webpackModules = parseBundleModules(webpackDistContent);
		this.compareModules(
			rspackModules.modules,
			webpackModules.modules,
			this.options.modules
		);
		this.compareModules(
			rspackModules.runtimeModules,
			webpackModules.runtimeModules,
			this.options.runtimeModules
		);
	}

	private compareModules(
		rspackModules: Map<string, string>,
		webpackModules: Map<string, string>,
		moduleList: string[] | boolean
	) {
		if (moduleList === true) {
			const modules = [
				...rspackModules.keys(),
				...webpackModules.keys()
			].filter((i, idx, arr) => arr.indexOf(i) === idx);
			for (let file of modules) {
				const rspackModuleContent =
					rspackModules.has(file) && formatCode(rspackModules.get(file)!);
				const webpackModuleContent =
					webpackModules.has(file) && formatCode(webpackModules.get(file)!);
				expect(rspackModuleContent || webpackModuleContent).toBeTruthy();
				expect(rspackModuleContent).toEqual(webpackModuleContent);
			}
		} else if (Array.isArray(moduleList)) {
			for (let file of moduleList as string[]) {
				const rspackModuleContent =
					rspackModules.has(file) && formatCode(rspackModules.get(file)!);
				const webpackModuleContent =
					webpackModules.has(file) && formatCode(webpackModules.get(file)!);
				expect(rspackModuleContent || webpackModuleContent).toBeTruthy();
				expect(rspackModuleContent).toEqual(webpackModuleContent);
			}
		}
	}

	private readConfigFiles(root: string, files: string[]) {
		const configFile = files
			.map(i => path.resolve(root, i))
			.find(i => fs.existsSync(i));
		if (configFile) {
			return require(configFile);
		} else {
			return {};
		}
	}

	private setDefaultOptions<T extends ECompilerType>(
		options: TCompilerOptions<T>,
		type: T,
		src: string,
		dist: string
	) {
		// output options
		options.output ??= {};
		options.output.filename ??= OUTPUT_MAIN_FILE;
		options.output.chunkFilename ??= "[name].chunk.js";
		options.output.library ??= "commonjs2";
		// entry options
		options.entry = path.join(src, "./src/index.js");
		options.context = src;
		// production
		options.mode = "development";
		options.devtool = false;
		// optimization
		options.optimization = {
			chunkIds: "named",
			moduleIds: "named"
		};
		if (type === ECompilerType.Webpack) {
			const webpackOptions = options as TCompilerOptions<ECompilerType.Webpack>;
			webpackOptions.plugins ??= [];
			webpackOptions.plugins!.push(createModulePlaceholderPlugin());

			webpackOptions.output!.pathinfo = false;
			webpackOptions.output!.environment = {
				arrowFunction: false,
				bigIntLiteral: false,
				const: false,
				destructuring: false,
				dynamicImport: false,
				// dynamicImportInWorker: false,
				forOf: false,
				// globalThis: false,
				module: false,
				optionalChaining: false,
				templateLiteral: false
			};
		}
		if (type === ECompilerType.Rspack) {
			const rspackOptions = options as TCompilerOptions<ECompilerType.Rspack>;
			rspackOptions.experiments = {
				rspackFuture: {
					disableTransformByDefault: true
				}
			};
		}
		return options;
	}
}
