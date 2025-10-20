import { NodeRunner, WebRunner } from "../runner";
import type {
	ECompilerType,
	ITestContext,
	ITestEnv,
	ITestRunner,
	TCompilerOptions,
	TCompilerStatsCompilation
} from "../type";
import { getCompiler } from "./common";

export type THotStepRuntimeLangData = {
	outdatedModules: string[];
	outdatedDependencies: Record<string, string[]>;

	updatedModules: string[];
	updatedRuntime: string[];

	acceptedModules: string[];
	disposedModules: string[];
};

export type THotStepRuntimeData = {
	javascript: THotStepRuntimeLangData;
	css: THotStepRuntimeLangData;
	statusPath: string[];
};

export function cachedStats<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string
): () => TCompilerStatsCompilation<T> {
	const compiler = context.getCompiler<T>(name);
	const statsGetter = (() => {
		let cached: TCompilerStatsCompilation<T> | null = null;
		return () => {
			if (cached) {
				return cached;
			}
			cached = compiler.getStats()!.toJson({
				errorDetails: true
			});
			return cached;
		};
	})();
	return statsGetter;
}

export function createRunner<T extends ECompilerType = ECompilerType.Rspack>(
	context: ITestContext,
	name: string,
	file: string,
	env: ITestEnv
): ITestRunner {
	const compiler = getCompiler(context, name);
	const testConfig = context.getTestConfig();
	const compilerOptions = compiler.getOptions() as TCompilerOptions<T>;
	const runnerOptions = {
		runInNewContext: false,
		cachable: true,
		env,
		stats: cachedStats(context, name),
		name,
		testConfig: context.getTestConfig(),
		source: context.getSource(),
		dist: context.getDist(),
		compilerOptions
	};
	if (
		compilerOptions.target === "web" ||
		compilerOptions.target === "webworker"
	) {
		return new WebRunner<T>({
			...runnerOptions,
			runInNewContext: true,
			location: testConfig.location || "https://test.cases/path/index.html"
		});
	}
	return new NodeRunner<T>(runnerOptions);
}

function getFileIndexHandler(
	context: ITestContext,
	name: string,
	file: string
) {
	const multiFileIndexMap: Record<string, number[]> =
		context.getValue(name, "multiFileIndexMap") || {};
	const runned = (context.getValue(name, "runned") as Set<string>) || new Set();
	if (typeof multiFileIndexMap[file] === "undefined") {
		throw new Error("Unexpect file in multiple runner");
	}
	const indexList = multiFileIndexMap[file];
	const seq = indexList.findIndex(
		(index, n) => !runned.has(`${name}:${file}[${n}]`)
	);
	if (seq === -1) {
		throw new Error(`File ${file} should run only ${indexList.length} times`);
	}
	const getIndex = () => [indexList[seq], seq];
	const flagIndex = () => runned.add(`${name}:${file}[${seq}]`);
	context.setValue(name, "runned", runned);
	return { getIndex, flagIndex };
}

export function getMultiCompilerRunnerKey(
	context: ITestContext,
	name: string,
	file: string
): string {
	const { getIndex } = getFileIndexHandler(context, name, file);
	const [index, seq] = getIndex();
	return `${name}-${index}[${seq}]`;
}

export function createMultiCompilerRunner<
	T extends ECompilerType = ECompilerType.Rspack
>(
	context: ITestContext,
	name: string,
	file: string,
	env: ITestEnv
): ITestRunner {
	const testConfig = context.getTestConfig();
	const { getIndex, flagIndex } = getFileIndexHandler(context, name, file);
	const multiCompilerOptions: TCompilerOptions<T>[] =
		context.getValue(name, "multiCompilerOptions") || [];
	const [index] = getIndex();
	const compilerOptions = multiCompilerOptions[index];
	let runner;
	const runnerOptions = {
		runInNewContext: false,
		cachable: true,
		env,
		stats: () => {
			const s = cachedStats(context, name)();
			if (s.children?.length && s.children.length > 1) {
				s.__index__ = index;
				return s;
			}
			return s.children![index];
		},
		name,
		testConfig: context.getTestConfig(),
		source: context.getSource(),
		dist: context.getDist(),
		compilerOptions
	};
	if (
		compilerOptions.target === "web" ||
		compilerOptions.target === "webworker"
	) {
		runner = new WebRunner<T>({
			...runnerOptions,
			runInNewContext: true,
			location: testConfig.location || "https://test.cases/path/index.html"
		});
	} else {
		runner = new NodeRunner<T>(runnerOptions);
	}
	flagIndex();
	return runner;
}
