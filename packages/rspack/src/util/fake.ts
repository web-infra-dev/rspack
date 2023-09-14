import { Compilation, Assets } from "..";
import * as tapable from "tapable";
import MergeCaller from "./MergeCaller";

export const createFakeProcessAssetsHook = (compilation: Compilation) => {
	type FakeProcessAssetsOptions = string | { name: string; stage?: number };

	const createFakeTap = (
		options: FakeProcessAssetsOptions,
		// @ts-expect-error
		fn,
		tap: string
	) => {
		if (typeof options === "string") options = { name: options };
		const hook = compilation.__internal_getProcessAssetsHookByStage(
			options.stage ?? 0
		);
		// @ts-expect-error
		hook[tap](options.name, fn);
	};
	return {
		name: "processAssets",
		tap: (options: FakeProcessAssetsOptions, fn: (assets: Assets) => void) =>
			createFakeTap(options, fn, "tap"),
		tapAsync: (
			options: FakeProcessAssetsOptions,
			fn: (assets: Assets, cb: tapable.InnerCallback<Error, void>) => void
		) => createFakeTap(options, fn, "tapAsync"),
		tapPromise: (
			options: FakeProcessAssetsOptions,
			fn: (assets: Assets) => Promise<void>
		) => createFakeTap(options, fn, "tapPromise"),
		stageAdditional: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stagePreProcess: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageDerived: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageAdditions: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageNone: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimize: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeCount: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeCompatibility: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeSize: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageDevTooling: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeInline: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageSummarize: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeHash: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeTransfer: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageAnalyse: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageReport: new tapable.AsyncSeriesHook<Assets>(["assets"])
	};
};

export type FakeHook<T> = T & { _fakeHook: true };

export const createFakeHook = <T extends object>(fakeHook: T): FakeHook<T> => {
	return Object.freeze(Object.assign(fakeHook, { _fakeHook: true as const }));
};

export const createProcessAssetsHook = <T>(
	processAssetsHooks: ReturnType<typeof createFakeProcessAssetsHook>,
	name: string,
	stage: number,
	getArgs: () => tapable.AsArray<T>
): FakeHook<
	Pick<tapable.AsyncSeriesHook<T>, "tap" | "tapAsync" | "tapPromise" | "name">
> => {
	const errorMessage = (
		reason: string
	) => `Can't automatically convert plugin using Compilation.hooks.${name} to Compilation.hooks.processAssets because ${reason}.
BREAKING CHANGE: Asset processing hooks in Compilation has been merged into a single Compilation.hooks.processAssets hook.`;
	const getOptions = (
		options: string | (tapable.TapOptions & { name: string })
	) => {
		if (typeof options === "string") options = { name: options };
		if (options.stage) {
			throw new Error(errorMessage("it's using the 'stage' option"));
		}
		return { ...options, stage: stage };
	};
	const tap: tapable.AsyncSeriesHook<T>["tap"] = (options, fn) => {
		processAssetsHooks.tap(getOptions(options), () => fn(...getArgs()));
	};
	const tapAsync: tapable.AsyncSeriesHook<T>["tapAsync"] = (options, fn) => {
		processAssetsHooks.tapAsync(getOptions(options), (assets, callback) =>
			(fn as any)(...getArgs(), callback)
		);
	};
	const tapPromise: tapable.AsyncSeriesHook<T>["tapPromise"] = (
		options,
		fn
	) => {
		processAssetsHooks.tapPromise(getOptions(options), () => fn(...getArgs()));
	};
	return createFakeHook({
		name,
		intercept() {
			throw new Error(errorMessage("it's using 'intercept'"));
		},
		tap,
		tapAsync,
		tapPromise
	});
};

export function createFakeCompilationDependencies(
	getDeps: () => string[],
	addDeps: (deps: string[]) => void
) {
	const addDepsCaller = new MergeCaller(addDeps, 10);
	return {
		*[Symbol.iterator]() {
			const deps = getDeps();
			for (const dep of deps) {
				yield dep;
			}
		},
		has(dep: string): boolean {
			return getDeps().includes(dep);
		},
		add: (dep: string) => {
			addDepsCaller.push(dep);
		},
		addAll: (deps: Iterable<string>) => {
			addDepsCaller.push(...deps);
		}
	};
}
