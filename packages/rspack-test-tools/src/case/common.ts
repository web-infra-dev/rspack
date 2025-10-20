import fs from "fs-extra";
import path from "path";
import merge from "webpack-merge";
import { readConfigFile } from "../helper";
import { normalizePlaceholder } from "../helper/expect/placeholder";
import checkArrayExpectation from "../helper/legacy/checkArrayExpectation";
import {
	ECompilerType,
	type ITestCompilerManager,
	type ITestContext,
	type ITestEnv,
	type TCompilation,
	type TCompiler,
	type TCompilerOptions,
	type TCompilerStats
} from "../type";

export function getCompiler<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string
) {
	return context.getCompiler(
		name,
		ECompilerType.Rspack
	) as ITestCompilerManager<T>;
}

export async function config<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string,
	configFiles: string[],
	defaultOptions: TCompilerOptions<T> = {}
): Promise<TCompilerOptions<T>> {
	const compiler = getCompiler<T>(context, name);
	compiler.setOptions(defaultOptions);
	if (Array.isArray(configFiles)) {
		const fileOptions = readConfigFile<T>(
			configFiles.map(i => context.getSource(i)),
			context,
			defaultOptions
		)[0];
		compiler.mergeOptions(fileOptions);
	}
	return compiler.getOptions() as TCompilerOptions<T>;
}

export async function compiler<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string
): Promise<TCompiler<T>> {
	const compiler = getCompiler(context, name);
	compiler.createCompiler();
	return compiler.getCompiler()! as TCompiler<T>;
}

export async function build<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string
): Promise<TCompiler<T>> {
	const compiler = getCompiler(context, name);
	await compiler.build();
	return compiler.getCompiler()! as TCompiler<T>;
}

export async function run<T extends ECompilerType = ECompilerType.Rspack>(
	env: ITestEnv,
	context: ITestContext,
	name: string,
	findBundle: (
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void
) {
	const testConfig = context.getTestConfig();
	if (testConfig.noTests) return;

	const compiler = getCompiler(context, name);
	if (typeof testConfig.beforeExecute === "function") {
		testConfig.beforeExecute(compiler.getOptions());
	}

	let bundles: string[] | void | string;
	if (testConfig.bundlePath) {
		bundles = testConfig.bundlePath;
	} else if (typeof findBundle === "function") {
		bundles = findBundle(context, compiler.getOptions() as TCompilerOptions<T>);
	} else {
		bundles = [];
	}

	if (typeof bundles === "string") {
		bundles = [bundles];
	}
	if (!bundles || !bundles.length) {
		return;
	}

	for (const bundle of bundles!) {
		if (!bundle) {
			continue;
		}
		const runner = context.getRunner(name, bundle, env);
		const mod = runner.run(bundle);
		const result =
			context.getValue<Array<Promise<unknown>>>(name, "modules") || [];
		result.push(mod);
		context.setValue<Array<Promise<unknown>>>(name, "modules", result);
	}

	const results =
		context.getValue<Array<Promise<unknown>>>(name, "modules") || [];
	await Promise.all(results);
}

export async function check<T extends ECompilerType = ECompilerType.Rspack>(
	env: ITestEnv,
	context: ITestContext,
	name: string
) {
	const testConfig = context.getTestConfig();
	if (testConfig.noTests) return;

	const compiler = getCompiler(context, name);

	const errors: Array<{ message: string; stack?: string }> = (
		context.getError(name) || []
	).map(e => ({
		message: e.message,
		stack: e.stack
	}));
	const warnings: Array<{ message: string; stack?: string }> = [];

	const stats = compiler.getStats();
	const options = compiler.getOptions() as TCompilerOptions<T>;
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
		context.clearError(name);
	}
}

export async function checkSnapshot<
	T extends ECompilerType = ECompilerType.Rspack
>(
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

	const compilerManager = getCompiler(context, name);
	const stats = compilerManager.getStats();
	const compiler = compilerManager.getCompiler();
	if (!stats || !compiler) return;

	const compilers: TCompiler<T>[] =
		"compilers" in compiler
			? (compiler.compilers as unknown as TCompiler<T>[])
			: [compiler as unknown as TCompiler<T>];
	const totalStats: TCompilerStats<T>[] =
		"stats" in stats
			? (stats.stats as unknown as TCompilerStats<T>[])
			: [stats as TCompilerStats<T>];
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
			(c as unknown as TCompiler<ECompilerType.Rspack>)._lastCompilation ||
			(
				c as unknown as TCompiler<ECompilerType.Webpack> & {
					_lastCompilation: TCompilation<T>;
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

				return `\`\`\`${tag} title=${file}\n${content}\n\`\`\``;
			});
		fileContents.sort();
		const content = fileContents.join("\n\n");
		const snapshotPath = path.isAbsolute(snapshot)
			? snapshot
			: path.resolve(
					context.getSource(),
					`./__snapshots__/${snapshot}${total > 1 ? `-${i}` : ""}`
				);

		env.expect(content).toMatchFileSnapshot(snapshotPath);
	}
}

export async function afterExecute(context: ITestContext, name: string) {
	const compiler = getCompiler(context, name);
	const testConfig = context.getTestConfig();
	if (typeof testConfig.afterExecute === "function") {
		let options = compiler.getOptions();
		if (Array.isArray(options) && options.length === 1) {
			options = options[0];
		}
		testConfig.afterExecute(options);
	}
}

export function findMultiCompilerBundle<
	T extends ECompilerType = ECompilerType.Rspack
>(
	context: ITestContext,
	name: string,
	multiFindBundle: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => string[] | string | void
) {
	if (typeof multiFindBundle !== "function") {
		return [];
	}

	const multiCompilerOptions = (context.getValue(
		name,
		"multiCompilerOptions"
	) || []) as TCompilerOptions<T>[];
	const result: string[] = [];
	const multiFileIndexMap: Record<string, number[]> =
		context.getValue(name, "multiFileIndexMap") || {};
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

	context.setValue(name, "multiFileIndexMap", multiFileIndexMap);

	return result;
}

export function configMultiCompiler<
	T extends ECompilerType = ECompilerType.Rspack
>(
	context: ITestContext,
	name: string,
	configFiles: string[],
	defaultOptions: (index: number, context: ITestContext) => TCompilerOptions<T>,
	overrideOptions: (
		index: number,
		context: ITestContext,
		options: TCompilerOptions<T>
	) => void
) {
	const multiCompilerOptions: TCompilerOptions<T>[] = [];
	const caseOptions: TCompilerOptions<T>[] = Array.isArray(configFiles)
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

							return c(options.env, options) as TCompilerOptions<T>;
						}

						return c as TCompilerOptions<T>;
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

	const compiler = getCompiler(context, name);
	compiler.setOptions(multiCompilerOptions as any);
	context.setValue(name, "multiCompilerOptions", multiCompilerOptions);
}
