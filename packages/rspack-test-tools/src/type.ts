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

export interface ITestContext {
	errors: Error[];
	getSource(sub?: string): string;
	getDist(sub?: string): string;
	options<T extends ECompilerType>(
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T> | void,
		name?: string
	): void;
	compiler<T extends ECompilerType>(
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | null,
		name?: string
	): void;
	stats<T extends ECompilerType>(
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | void,
		name?: string
	): void;
	result<T extends ECompilerType>(
		fn: <R>(compiler: TCompiler<T> | null, result: R) => R,
		name?: string
	): void;
	build<T extends ECompilerType>(
		fn: (compiler: TCompiler<T>) => Promise<void>,
		name?: string
	): Promise<void>;
	emitError(err: Error | string): void;
	hasError(): boolean;
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
	options(
		context: ITestContext,
		fn: (options: TCompilerOptions<T>) => TCompilerOptions<T> | void
	): void;
	compiler(
		context: ITestContext,
		fn: (
			options: TCompilerOptions<T>,
			compiler: TCompiler<T> | null
		) => TCompiler<T> | null
	): void;
	stats(
		context: ITestContext,
		fn: (
			compiler: TCompiler<T> | null,
			stats: TCompilerStats<T> | null
		) => TCompilerStats<T> | void
	): void;
	result(
		context: ITestContext,
		fn: <R>(compiler: TCompiler<T> | null, result: R) => R
	): void;
	build(
		context: ITestContext,
		fn: (compiler: TCompiler<T>) => Promise<void>
	): Promise<void>;
}

export interface ITestLoader {
	walk(): Promise<void>;
}

export interface ITesterConfig {
	name: string;
	src: string;
	dist: string;
	steps?: ITestProcessor[];
}

export interface ITester {
	step: number;
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
