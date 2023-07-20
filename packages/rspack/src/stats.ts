/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/tree/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/stats
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */
import * as binding from "@rspack/binding";
import { Compilation } from ".";
import { StatsValue, StatsOptions } from "./config";
import type { StatsCompilation } from "./stats/statsFactoryUtils";

export class Stats {
	#inner: binding.JsStats;
	compilation: Compilation;

	constructor(compilation: Compilation) {
		this.#inner = compilation.__internal_getInner().getStats();
		this.compilation = compilation;
	}

	get hash() {
		return this.compilation.hash;
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

		return statsFactory.create("compilation", this.compilation, {
			compilation: this.compilation,
			_inner: this.#inner
		});
	}

	toString(opts?: StatsValue) {
		const options = this.compilation.createStatsOptions(opts, {
			forToString: true
		});
		const statsFactory = this.compilation.createStatsFactory(options);

		const statsPrinter = this.compilation.createStatsPrinter(options);

		const data = statsFactory.create("compilation", this.compilation, {
			compilation: this.compilation,
			_inner: this.#inner
		});

		const result = statsPrinter.print("compilation", data);

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
	const pn = (typeof name === "string" && name.toLowerCase()) || name;
	switch (pn) {
		case "none":
			return {
				all: false
			};
		case "verbose":
			return {
				all: true
			};
		case "errors-only":
			return {
				all: false,
				errors: true,
				errorsCount: true
				// TODO: moduleTrace: true,
				// TODO: logging: "error"
			};
		case "errors-warnings":
			return {
				all: false,
				errors: true,
				errorsCount: true,
				warnings: true,
				warningsCount: true
				// TODO: logging: "warn"
			};
		default:
			return {};
	}
}
