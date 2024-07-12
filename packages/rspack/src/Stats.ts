/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import type * as binding from "@rspack/binding";

import type { Compilation } from "./Compilation";
import type { StatsOptions, StatsValue } from "./config";
import type { StatsCompilation } from "./stats/statsFactoryUtils";

export type {
	StatsAsset,
	StatsChunk,
	StatsCompilation,
	StatsError,
	StatsModule,
	StatsWarnings
} from "./stats/statsFactoryUtils";

export class Stats {
	#inner: binding.JsStats;
	compilation: Compilation;
	#innerMap: WeakMap<Compilation, binding.JsStats>;

	constructor(compilation: Compilation) {
		this.#inner = compilation.__internal_getInner().getStats();
		this.compilation = compilation;
		this.#innerMap = new WeakMap([[this.compilation, this.#inner]]);
	}

	// use correct JsStats for child compilation
	#getInnerByCompilation(compilation: Compilation) {
		if (this.#innerMap.has(compilation)) {
			return this.#innerMap.get(compilation);
		}
		const inner = compilation.__internal_getInner().getStats();
		this.#innerMap.set(compilation, inner);
		return inner;
	}

	get hash() {
		return this.compilation.hash;
	}

	get startTime() {
		return this.compilation.startTime;
	}

	get endTime() {
		return this.compilation.endTime;
	}

	hasErrors() {
		return this.#inner.getErrors().length > 0;
	}

	hasWarnings() {
		return this.#inner.getWarnings().length > 0;
	}

	toJson(opts?: StatsValue, forToString?: boolean): StatsCompilation {
		const options = this.compilation.createStatsOptions(opts, {
			forToString
		});

		const statsFactory = this.compilation.createStatsFactory(options);

		// FIXME: This is a really ugly workaround for avoid panic for accessing previous compilation.
		// Modern.js dev server will detect whether the returned stats is available.
		// So this does not do harm to these frameworks.
		// Modern.js: https://github.com/web-infra-dev/modern.js/blob/63f916f882f7d16096949e264e119218c0ab8d7d/packages/server/server/src/dev-tools/dev-middleware/socketServer.ts#L172
		let stats: StatsCompilation | null = null;
		try {
			stats = statsFactory.create("compilation", this.compilation, {
				compilation: this.compilation,
				getInner: this.#getInnerByCompilation.bind(this)
			});
		} catch (e) {
			console.warn(
				"Failed to get stats. " +
					"Are you trying to access the stats from the previous compilation?"
			);
		}
		return stats as StatsCompilation;
	}

	toString(opts?: StatsValue) {
		const options = this.compilation.createStatsOptions(opts, {
			forToString: true
		});
		const statsFactory = this.compilation.createStatsFactory(options);

		const statsPrinter = this.compilation.createStatsPrinter(options);

		// FIXME: This is a really ugly workaround for avoid panic for accessing previous compilation.
		// Modern.js dev server will detect whether the returned stats is available.
		// So this does not do harm to these frameworks.
		// Modern.js: https://github.com/web-infra-dev/modern.js/blob/63f916f882f7d16096949e264e119218c0ab8d7d/packages/server/server/src/dev-tools/dev-middleware/socketServer.ts#L172
		let stats: StatsCompilation | null = null;
		try {
			stats = statsFactory.create("compilation", this.compilation, {
				compilation: this.compilation,
				getInner: this.#getInnerByCompilation.bind(this)
			});
		} catch (e) {
			console.warn(
				"Failed to get stats. " +
					"Are you trying to access the stats from the previous compilation?"
			);
		}

		if (!stats) {
			return "";
		}

		const result = statsPrinter.print("compilation", stats);

		return result === undefined ? "" : result;
	}
}

export function normalizeStatsPreset(options?: StatsValue): StatsOptions {
	if (typeof options === "boolean" || typeof options === "string")
		return presetToOptions(options);
	else if (!options) return {};
	else {
		let obj = { ...presetToOptions(options.preset), ...options };
		delete obj.preset;
		return obj;
	}
}

function presetToOptions(name?: boolean | string): StatsOptions {
	const preset = (typeof name === "string" && name.toLowerCase()) || name;
	switch (preset) {
		case "none":
			return {
				all: false
			};
		case "verbose":
			return {
				all: true,
				modulesSpace: Infinity
			};
		case "errors-only":
			return {
				all: false,
				errors: true,
				errorsCount: true,
				logging: "error",
				moduleTrace: true
			};
		case "errors-warnings":
			return {
				all: false,
				errors: true,
				errorsCount: true,
				warnings: true,
				warningsCount: true,
				logging: "warn"
			};
		default:
			return {};
	}
}
