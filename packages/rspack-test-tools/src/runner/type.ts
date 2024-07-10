import type { ITestEnv } from "../type";

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
	[key: string]: any;
}

export interface IBasicGlobalContext {
	console: Console;
	setTimeout: typeof setTimeout;
	clearTimeout: typeof clearTimeout;
	[key: string]: any;
}

export type TModuleObject = { exports: unknown };

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
