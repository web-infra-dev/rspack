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
	Unknown = 0,
	Evaluated = 1,
	Unlinked = 2
}

export interface IBasicModuleScope extends ITestEnv {
	console: Record<string, (...args: any[]) => void>;
	expect: jest.Expect;
	[key: string]: any;
}

export interface IBasicGlobalContext {
	console: Record<string, (...args: any[]) => void>;
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
