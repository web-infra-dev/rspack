import {
	ECompilerType,
	ITestEnv,
	TCompilerOptions,
	TCompilerStats,
	TTestConfig
} from "../type";

export type TRunnerRequirer = (
	currentDirectory: string,
	modulePath: string[] | string,
	context?: {
		file?: TBasicRunnerFile;
		esmMode?: EEsmMode;
	}
) => Object | Promise<Object>;

export type TBasicRunnerFile = {
	path: string;
	content: string;
	subPath: string;
};

export enum EEsmMode {
	Unknown,
	Evaluated,
	Unlinked
}

export interface IBasicModuleScope extends ITestEnv {
	console: Console;
	expect: jest.Expect;
	jest: typeof jest;
	[key: string]: any;
}

export interface IBasicGlobalContext {
	console: Console;
	expect: jest.Expect;
	setTimeout: typeof setTimeout;
	clearTimeout: typeof clearTimeout;
	[key: string]: any;
}

export interface IBasicRunnerOptions<T extends ECompilerType> {
	env: ITestEnv;
	stats?: TCompilerStats<T>;
	name: string;
	runInNewContext: boolean;
	testConfig: TTestConfig<T>;
	source: string;
	dist: string;
	compilerOptions: TCompilerOptions<T>;
}
