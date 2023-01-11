import { Compilation, Assets } from "..";
import * as tapable from "tapable";

export function mapValues(
	record: Record<string, string>,
	fn: (key: string) => string
) {
	return Object.fromEntries(
		Object.entries(record).map(([key, value]) => [key, fn(value)])
	);
}

export function isNil(value: unknown): value is null | undefined {
	return value === null || value === undefined;
}

export function isPromiseLike(value: unknown): value is Promise<any> {
	return (
		typeof value === "object" &&
		value !== null &&
		typeof (value as any).then === "function"
	);
}

export const createProcessAssetsFakeHook = (compilation: Compilation) => {
	type FakeProcessAssetsOptions = string | { name: string; stage?: number };

	const createFakeTap = (
		options: FakeProcessAssetsOptions,
		fn,
		tap: string
	) => {
		if (typeof options === "string") options = { name: options };
		const hook = compilation.__internal_getProcessAssetsHookByStage(
			options.stage ?? 0
		);
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
		stageNone: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageOptimizeInline: new tapable.AsyncSeriesHook<Assets>(["assets"]),
		stageSummarize: new tapable.AsyncSeriesHook<Assets>(["assets"]),
	};
};
