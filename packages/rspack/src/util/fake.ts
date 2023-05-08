import { Compilation, Assets } from "..";
import * as tapable from "tapable";

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
		stageAdditions: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageNone: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeInline: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageSummarize: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeHash: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageReport: new tapable.AsyncSeriesHook<Assets>(["assets"])
	};
};

export function createFakeCompilationDependencies(
	deps: string[],
	addDeps: (deps: string[]) => void
) {
	return {
		*[Symbol.iterator]() {
			for (const dep of deps) {
				yield dep;
			}
		},
		has(dep: string): boolean {
			return deps.includes(dep);
		},
		add: (dep: string) => {
			addDeps([dep]);
		},
		addAll: (deps: Iterable<string>) => {
			addDeps(Array.from(deps));
		}
	};
}
