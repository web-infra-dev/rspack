/// <reference types="../jest.d.ts" />

import {
	Compiler as RspackCompiler,
	RspackOptions,
	Stats as RspackStats,
	StatsCompilation as RspackStatsCompilation
} from "@rspack/core";
import EventEmitter from "events";
import type {
	Compiler as WebpackCompiler,
	Configuration as WebpackOptions,
	Stats as WebpackStats,
	StatsCompilation as WebpackStatsCompilation
} from "webpack";

import { IBasicModuleScope, TRunnerRequirer } from "./runner/type";

export interface ITestContext {
	getSource(sub?: string): string;
	getDist(sub?: string): string;
	getTemp(sub?: string): string | null;

	getCompiler<T extends ECompilerType>(
		name: string,
		type: T | void
	): ITestCompilerManager<T>;

	getTestConfig<T extends ECompilerType>(): TTestConfig<T>;
	getRunnerFactory<T extends ECompilerType>(
		name: string
	): TRunnerFactory<T> | null;
	getRunner(key: string): ITestRunner | null;
	setRunner(key: string, runner: ITestRunner): void;

	setValue<T>(name: string, key: string, value: T): void;
	getValue<T>(name: string, key: string): T | void;
	getNames(): string[];

	hasError(name?: string): boolean;
	emitError(name: string, err: Error | string): void;
	getError(name?: string): Error[];
	clearError(name?: string): void;
}

export enum ECompilerType {
	Rspack = "rspack",
	Webpack = "webpack"
}

export type TCompilerOptions<T> = T extends ECompilerType.Rspack
	? RspackOptions
	: WebpackOptions;
export type TCompiler<T> = T extends ECompilerType.Rspack
	? RspackCompiler
	: WebpackCompiler;
export type TCompilerStats<T> = T extends ECompilerType.Rspack
	? RspackStats
	: WebpackStats;

export type TCompilerStatsCompilation<T> = T extends ECompilerType.Rspack
	? RspackStatsCompilation
	: WebpackStatsCompilation;

export interface ITestCompilerManager<T extends ECompilerType> {
	getOptions(): TCompilerOptions<T>;
	setOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T>;
	mergeOptions(newOptions: TCompilerOptions<T>): TCompilerOptions<T>;
	getCompiler(): TCompiler<T> | null;
	createCompiler(): TCompiler<T>;
	build(): Promise<TCompilerStats<T>>;
	watch(timeout?: number): void;
	getStats(): TCompilerStats<T> | null;
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
	testConfig?: TTestConfig<ECompilerType>;
	compilerFactories?: TCompilerFactories;
	runnerFactory?: new (
		name: string,
		context: ITestContext
	) => TRunnerFactory<ECompilerType>;
}

export interface ITester {
	step: number;
	total: number;
	getContext(): ITestContext;
	prepare(): Promise<void>;
	compile(): Promise<void>;
	check(env: ITestEnv): Promise<void>;
	next(): boolean;
	resume(): Promise<void>;
}

export interface ITestProcessor {
	beforeAll?(context: ITestContext): Promise<void>;
	afterAll?(context: ITestContext): Promise<void>;
	before?(context: ITestContext): Promise<void>;
	after?(context: ITestContext): Promise<void>;

	config?(context: ITestContext): Promise<void>;
	compiler?(context: ITestContext): Promise<void>;
	build?(context: ITestContext): Promise<void>;
	run?(env: ITestEnv, context: ITestContext): Promise<void>;
	check?(env: ITestEnv, context: ITestContext): Promise<unknown>;
}

export interface ITestReporter<T> {
	init(data?: T): Promise<void>;
	increment(id: string, data: T): Promise<void>;
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
	data: Array<TDiffStatsItem>;
};

export interface ITestEnv {
	expect: jest.Expect;
	it: (...args: any[]) => void;
	beforeEach: (...args: any[]) => void;
	afterEach: (...args: any[]) => void;
	[key: string]: unknown;
}

export type TTestConfig<T extends ECompilerType> = {
	validate?: (stats: TCompilerStats<T>, stderr?: string) => void;
	noTest?: boolean;
	beforeExecute?: () => void;
	afterExecute?: () => void;
	moduleScope?: (ms: IBasicModuleScope) => IBasicModuleScope;
	findBundle?: (
		index: number,
		options: TCompilerOptions<T>
	) => string | string[];
	bundlePath?: string[];
	nonEsmThis?: (p: string | string[]) => Object;
	modules?: Record<string, Object>;
	timeout?: number;
};

export type TTestFilter<T extends ECompilerType> = (
	creatorConfig: Record<string, unknown>,
	testConfig: TTestConfig<T>
) => boolean | string;

export interface ITestRunner {
	run(file: string): Promise<unknown>;
	getRequire(): TRunnerRequirer;
	getGlobal(name: string): unknown;
}

export type TCompilerFactory<T extends ECompilerType> = (
	options: TCompilerOptions<T> | TCompilerOptions<T>[]
) => TCompiler<T>;

export interface TRunnerFactory<T extends ECompilerType> {
	create(
		file: string,
		compilerOptions: TCompilerOptions<T>,
		env: ITestEnv
	): ITestRunner;
}

export type TUpdateOptions = {
	updateIndex: number;
	totalUpdates: number;
	changedFiles: string[];
};

export type TCompilerFactories = Record<
	ECompilerType,
	TCompilerFactory<ECompilerType>
>;
