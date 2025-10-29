import type { Compilation, Compiler, RspackOptions, Stats } from "@rspack/core";
import fs from "fs-extra";
import path from "path";
import merge from "webpack-merge";
import { readConfigFile } from "../helper";
import { normalizePlaceholder } from "../helper/expect/placeholder";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import { DEBUG_SCOPES } from "../test/debug";
import type { ITestContext, ITestEnv } from "../type";

export async function config(
	context: ITestContext,
	name: string,
	configFiles: string[],
	defaultOptions: RspackOptions = {}
): Promise<RspackOptions> {
	const compiler = context.getCompiler();
	compiler.setOptions(defaultOptions);
	if (Array.isArray(configFiles)) {
		const fileOptions = readConfigFile(
			configFiles.map(i => context.getSource(i)),
			context,
			defaultOptions
		)[0];
		compiler.mergeOptions(fileOptions);
	}
	return compiler.getOptions() as RspackOptions;
}

export async function compiler(
	context: ITestContext,
	name: string
): Promise<Compiler> {
	const compiler = context.getCompiler();
	compiler.createCompiler();
	return compiler.getCompiler()! as Compiler;
}

export async function build(
	context: ITestContext,
	name: string
): Promise<Compiler> {
	const compiler = context.getCompiler();
	await compiler.build();
	return compiler.getCompiler()! as Compiler;
}

export async function run(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	findBundle: (
		context: ITestContext,
		options: RspackOptions
	) => string[] | string | void
) {
	const testConfig = context.getTestConfig();
	if (testConfig.noTests) return;

	const compiler = context.getCompiler();
	if (typeof testConfig.beforeExecute === "function") {
		testConfig.beforeExecute(compiler.getOptions());
	}

	let bundles: string[] | void | string;
	if (typeof findBundle === "function") {
		bundles = findBundle(context, compiler.getOptions() as RspackOptions);
	} else {
		bundles = [];
	}

	if (typeof bundles === "string") {
		bundles = [bundles];
	}

	if (__DEBUG__) {
		context.setValue(DEBUG_SCOPES.RunFindBundle, bundles);
	}

	if (!bundles || !bundles.length) {
		return;
	}

	if (__DEBUG__) {
		context.setValue(DEBUG_SCOPES.RunLogs, []);
		context.setValue(DEBUG_SCOPES.RunErrors, []);
	}

	for (const bundle of bundles!) {
		if (!bundle) {
			continue;
		}
		const runner = context.getRunner(bundle, env);
		if (__DEBUG__) {
			const runLogs = context.getValue(DEBUG_SCOPES.RunLogs) as
				| string[]
				| undefined;
			runLogs?.push(
				`Start running entry: ${bundle} in ${runner.constructor.name}(${(runner as any).__key__})`
			);
		}
		const mod = runner.run(bundle);
		const result = context.getValue<Array<Promise<unknown>>>("modules") || [];
		result.push(mod);
		context.setValue<Array<Promise<unknown>>>("modules", result);
	}

	const results = context.getValue<Array<Promise<unknown>>>("modules") || [];
	await Promise.all(results);
}

export async function check(
	env: ITestEnv,
	context: ITestContext,
	name: string
) {
	const testConfig = context.getTestConfig();
	if (testConfig.noTests) return;

	const compiler = context.getCompiler();

	const errors: Array<{ message: string; stack?: string }> = (
		context.getError() || []
	).map(e => ({
		message: e.message,
		stack: e.stack
	}));
	const warnings: Array<{ message: string; stack?: string }> = [];

	const stats = compiler.getStats();
	const options = compiler.getOptions() as RspackOptions;
	if (stats) {
		if (testConfig.writeStatsOuptut) {
			fs.writeFileSync(
				path.join(context.getDist(), "stats.txt"),
				stats.toString({
					preset: "verbose",
					colors: false
				}),
				"utf-8"
			);
		}

		if (testConfig.writeStatsJson) {
			const jsonStats = stats.toJson({
				errorDetails: true
			});
			fs.writeFileSync(
				path.join(context.getDist(), "stats.json"),
				JSON.stringify(jsonStats, null, 2),
				"utf-8"
			);
		}

		if (
			fs.existsSync(context.getSource("errors.js")) ||
			fs.existsSync(context.getSource("warnings.js")) ||
			stats.hasErrors() ||
			stats.hasWarnings()
		) {
			const statsJson = stats.toJson({
				errorDetails: true
			});
			if (statsJson.errors) {
				errors.push(...statsJson.errors);
			}
			if (statsJson.warnings) {
				warnings.push(...statsJson.warnings);
			}
		}
	}
	await checkArrayExpectation(
		context.getSource(),
		{ errors },
		"error",
		"errors",
		"Error",
		options
	);

	await checkArrayExpectation(
		context.getSource(),
		{ warnings },
		"warning",
		"warnings",
		"Warning",
		options
	);

	// clear error if checked
	if (fs.existsSync(context.getSource("errors.js"))) {
		context.clearError();
	}
}

export async function checkSnapshot(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	snapshot: string,
	filter?: (file: string) => boolean
) {
	if (path.extname(snapshot) === ".snap") {
		throw new Error(
			"Snapshot with `.snap` will be managed by jest, please use `.snap.txt` instead"
		);
	}

	const compilerManager = context.getCompiler();
	const stats = compilerManager.getStats();
	const compiler = compilerManager.getCompiler();
	if (!stats || !compiler) return;

	const compilers: Compiler[] =
		"compilers" in compiler
			? (compiler.compilers as unknown as Compiler[])
			: [compiler as unknown as Compiler];
	const totalStats: Stats[] =
		"stats" in stats ? (stats.stats as unknown as Stats[]) : [stats as Stats];
	const total = compilers.length;
	for (let i = 0; i < compilers.length; i++) {
		const c = compilers[i];
		const stats = totalStats[i];
		if (stats.hasErrors()) {
			const errors = [];
			errors.push(...stats.compilation.errors);

			throw new Error(
				`Failed to compile in fixture ${name}, Errors: ${errors
					?.map(i => `${i.message}\n${i.stack}`)
					.join("\n\n")}`
			);
		}
		const compilation =
			(c as unknown as Compiler)._lastCompilation ||
			(
				c as unknown as Compiler & {
					_lastCompilation: Compilation;
				}
			)._lastCompilation;

		const snapshotFileFilter =
			filter ||
			((file: string) =>
				(file.endsWith(".js") || file.endsWith(".mjs")) &&
				!file.includes("runtime.js"));

		const fileContents = Object.entries(compilation.assets)
			.filter(([file]) => snapshotFileFilter(file))
			.map(([file, source]) => {
				const tag = path.extname(file).slice(1) || "txt";
				let content = normalizePlaceholder(source.source().toString());
				const testConfig = context.getTestConfig();
				if (testConfig.snapshotContent) {
					content = testConfig.snapshotContent(content);
				}
				const filePath = file.replaceAll(path.sep, "/");

				return `\`\`\`${tag} title=${filePath}\n${content}\n\`\`\``;
			});
		fileContents.sort();
		const content = fileContents.join("\n\n");
		const snapshotPath = path.isAbsolute(snapshot)
			? snapshot
			: path.resolve(
					context.getSource(),
					path.join("__snapshots__", `${snapshot}${total > 1 ? `-${i}` : ""}`)
				);

		env.expect(content).toMatchFileSnapshot(snapshotPath);
	}
}

export async function afterExecute(context: ITestContext, name: string) {
	const compiler = context.getCompiler();
	const testConfig = context.getTestConfig();
	if (typeof testConfig.afterExecute === "function") {
		let options = compiler.getOptions();
		if (Array.isArray(options) && options.length === 1) {
			options = options[0];
		}
		testConfig.afterExecute(options);
	}
}

export function findMultiCompilerBundle(
	context: ITestContext,
	name: string,
	multiFindBundle: (
		index: number,
		context: ITestContext,
		options: RspackOptions
	) => string[] | string | void
) {
	if (typeof multiFindBundle !== "function") {
		return [];
	}

	const multiCompilerOptions = (context.getValue("multiCompilerOptions") ||
		[]) as RspackOptions[];
	const result: string[] = [];
	const multiFileIndexMap: Record<string, number[]> =
		context.getValue("multiFileIndexMap") || {};
	for (const [index, compilerOptions] of multiCompilerOptions.entries()) {
		const curBundles = multiFindBundle!(index, context, compilerOptions);

		const bundles = Array.isArray(curBundles)
			? curBundles
			: curBundles
				? [curBundles]
				: [];

		for (const bundle of bundles) {
			if (multiFileIndexMap[bundle]) {
				multiFileIndexMap[bundle].push(index);
			} else {
				multiFileIndexMap[bundle] = [index];
			}
		}

		result.push(...bundles);
	}

	context.setValue("multiFileIndexMap", multiFileIndexMap);

	return result;
}

export function configMultiCompiler(
	context: ITestContext,
	name: string,
	configFiles: string[],
	defaultOptions: (index: number, context: ITestContext) => RspackOptions,
	overrideOptions: (
		index: number,
		context: ITestContext,
		options: RspackOptions
	) => void
) {
	const multiCompilerOptions: RspackOptions[] = [];
	const caseOptions: RspackOptions[] = Array.isArray(configFiles)
		? readConfigFile(
				configFiles!.map(i => context.getSource(i)),
				context,
				{},
				configs => {
					return configs.flatMap(c => {
						if (typeof c === "function") {
							const options = {
								testPath: context.getDist(),
								env: undefined
							};

							return c(options.env, options) as RspackOptions;
						}

						return c as RspackOptions;
					});
				}
			)
		: [{}];

	for (const [index, options] of caseOptions.entries()) {
		const compilerOptions = merge(
			typeof defaultOptions === "function"
				? defaultOptions!(index, context)
				: {},
			options
		);

		if (typeof overrideOptions === "function") {
			overrideOptions!(index, context, compilerOptions);
		}

		multiCompilerOptions.push(compilerOptions);
	}

	const compiler = context.getCompiler();
	compiler.setOptions(multiCompilerOptions as any);
	context.setValue("multiCompilerOptions", multiCompilerOptions);
}
