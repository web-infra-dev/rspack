import {
	RspackOptions,
	Compiler as RspackCompiler,
	Stats as RspackStats
} from "@rspack/core";
import type {
	Configuration as WebpackOptions,
	Compiler as WebpackCompiler,
	Stats as WebpackStats
} from "webpack";
import { IBasicModuleScope, TRunnerRequirer } from "./runner/type";
import EventEmitter from "events";

export interface ITestContext {
	getSource(sub?: string): string;
	getDist(sub?: string): string;
	getTemp(sub?: string): string | null;

	getCompiler<T extends ECompilerType>(
		name: string,
		factory?: TCompilerFactory<T>
	): ITestCompilerManager<T>;

	setResult<T>(name: string, value: T): void;
	getResult<T>(name: string): T | void;
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
}

export interface ITester {
	step: number;
	total: number;
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
	it: (...args: any[]) => void;
	beforeEach: (...args: any[]) => void;
	afterEach: (...args: any[]) => void;
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
}

export type TCompilerFactory<T extends ECompilerType> = (
	options: TCompilerOptions<T> | TCompilerOptions<T>[]
) => TCompiler<T>;
