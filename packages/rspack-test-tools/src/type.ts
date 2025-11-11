/// <reference types="../rstest.d.ts" />

import type EventEmitter from "node:events";
import type {
	Compiler,
	MultiStats,
	RspackOptions,
	Stats,
	StatsCompilation
} from "@rspack/core";

export interface ITestContext {
	getSource(sub?: string): string;
	getDist(sub?: string): string;
	getTemp(sub?: string): string | null;

	getCompiler(): ITestCompilerManager;
	closeCompiler(): Promise<void>;

	getTestConfig(): TTestConfig;
	getRunner(file: string, env: ITestEnv): ITestRunner;

	setValue<T>(key: string, value: T): void;
	getValue<T>(key: string): T | void;

	hasError(): boolean;
	emitError(err: Error | string): void;
	getError(): Error[];
	clearError(): void;
}

export interface ITestCompilerManager {
	getOptions(): RspackOptions;
	setOptions(newOptions: RspackOptions): RspackOptions;
	mergeOptions(newOptions: RspackOptions): RspackOptions;
	getCompiler(): Compiler | null;
	createCompiler(): Compiler;
	createCompilerWithCallback(
		callback: (error: Error | null, stats: Stats | null) => void
	): Compiler;
	build(): Promise<Stats>;
	watch(timeout?: number): void;
	getStats(): Stats | MultiStats | null;
	getEmitter(): EventEmitter;
	close(): Promise<void>;
}

export interface ITestLoader {
	walk(): Promise<void>;
}

export type TTestRunResult = Record<string, any>;

export interface ITesterConfig {
	name: string;
	src: string;
	dist: string;
	temp?: string;
	steps?: ITestProcessor[];
	testConfig?: TTestConfig;
	contextValue?: Record<string, unknown>;
	runnerCreator?: TTestRunnerCreator;
	createContext?: (config: ITesterConfig) => ITestContext;
}

export interface ITester {
	step: number;
	total: number;
	getContext(): ITestContext;
	prepare(): Promise<void>;
	compile(): Promise<void>;
	check(env: ITestEnv): Promise<void>;
	after(): Promise<void>;
	next(): boolean;
	resume(): Promise<void>;
}

export interface ITestProcessor {
	beforeAll?(context: ITestContext): Promise<void>;
	afterAll?(context: ITestContext): Promise<void>;
	before?(context: ITestContext): Promise<void>;
	after?(context: ITestContext): Promise<void>;

	config(context: ITestContext): Promise<void>;
	compiler(context: ITestContext): Promise<void>;
	build(context: ITestContext): Promise<void>;
	run(env: ITestEnv, context: ITestContext): Promise<void>;
	check(env: ITestEnv, context: ITestContext): Promise<unknown>;
}

export interface ITestReporter {
	init<T>(data?: T): Promise<void>;
	increment<T>(id: string, data: T): Promise<void>;
	failure(id: string): Promise<void>;
	output(): Promise<void>;
}

export enum ECompareResultType {
	Same = "same",
	Missing = "missing",
	OnlyDist = "only-dist",
	OnlySource = "only-source",
	Different = "different"
}
export type TCompareModules = string[] | true;
export type TCompareResult = {
	type: ECompareResultType;
	detail?: unknown;
	source?: string;
	dist?: string;
	lines?: {
		common: number;
		source: number;
		dist: number;
	};
};
export type TModuleCompareResult = TCompareResult & {
	name: string;
};

export type TFileCompareResult = TCompareResult & {
	file: {
		source: string;
		dist: string;
	};
	modules: Partial<
		Record<"modules" | "runtimeModules", TModuleCompareResult[]>
	>;
};

export type TDiffStatsItem = {
	name: string;
	source: string;
	dist: string;
	type: ECompareResultType;
};

export type TDiffStats = {
	root: string;
	data: TDiffStatsItem[];
};

export interface ITestEnv {
	expect: Expect;
	it: (...args: any[]) => void;
	beforeEach: (...args: any[]) => void;
	afterEach: (...args: any[]) => void;
	[key: string]: unknown;
}

export type TTestConfig = {
	location?: string;
	validate?: (stats: Stats | MultiStats, stderr?: string) => void;
	noTests?: boolean;
	writeStatsOuptut?: boolean;
	writeStatsJson?: boolean;
	beforeExecute?: (options: RspackOptions) => void;
	afterExecute?: (options: RspackOptions) => void;
	moduleScope?: (
		ms: IModuleScope,
		stats?: StatsCompilation,
		options?: RspackOptions
	) => IModuleScope;
	checkStats?: (
		stepName: string,
		jsonStats: StatsCompilation | undefined,
		stringStats: String
	) => boolean;
	findBundle?: (
		index: number,
		options: RspackOptions,
		stepName?: string
	) => string | string[];
	bundlePath?: string[];
	nonEsmThis?: (p: string | string[]) => Object;
	modules?: Record<string, Object>;
	timeout?: number;
	concurrent?: boolean;
	snapshotContent?(content: string): string;

	// Only valid for Hot tests
	checkSteps?: boolean;
	// Only valid for Watch tests
	ignoreNotFriendlyForIncrementalWarnings?: boolean;
	// Only valid for ESM library tests
	esmLibPluginOptions?: {
		preserveModules?: string;
	};
	resourceLoader?: (url: string, element: HTMLScriptElement) => Buffer | null;
};

export type TTestFilter = (
	creatorConfig: Record<string, unknown>,
	testConfig: TTestConfig
) => boolean | string;

export interface ITestRunner {
	run(file: string): Promise<unknown>;
	getRequire(): TRunnerRequirer;
	getGlobal(name: string): unknown;
}

export type TCompilerFactory = (
	options: RspackOptions | RspackOptions[],
	callback?: (error: Error | null, stats: Stats | null) => void
) => Compiler;

export interface TRunnerFactory {
	create(
		file: string,
		compilerOptions: RspackOptions,
		env: ITestEnv
	): ITestRunner;
}

export type THotUpdateContext = {
	updateIndex: number;
	totalUpdates: number;
	changedFiles: string[];
};

export type TRunnerRequirer = (
	currentDirectory: string,
	modulePath: string[] | string,
	context?: {
		file?: TRunnerFile;
		esmMode?: EEsmMode;
	}
) => Object | Promise<Object>;

export type TRunnerFile = {
	path: string;
	content: string;
	subPath: string;
};

export enum EEsmMode {
	Unknown = 0,
	Evaluated = 1,
	Unlinked = 2
}

export interface IModuleScope extends ITestEnv {
	console: Record<string, (...args: any[]) => void>;
	expect: Expect;
	[key: string]: any;
}

export interface IGlobalContext {
	console: Record<string, (...args: any[]) => void>;
	setTimeout: typeof setTimeout;
	clearTimeout: typeof clearTimeout;
	[key: string]: any;
}

export type TModuleObject = { exports: unknown };

export type TTestRunnerCreator = {
	key: (context: ITestContext, name: string, file: string) => string;
	runner: (
		context: ITestContext,
		name: string,
		file: string,
		env: ITestEnv
	) => ITestRunner;
};

declare global {
	var __DEBUG__: boolean;
	var __TEST_PATH__: string;
	var __TEST_FIXTURES_PATH__: string;
	var __TEST_DIST_PATH__: string;
	var __ROOT_PATH__: string;
	var __RSPACK_PATH__: string;
	var __RSPACK_TEST_TOOLS_PATH__: string;
}
