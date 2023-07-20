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
import { Compilation } from ".";
import { StatsValue, StatsOptions } from "./config";
import type { StatsCompilation } from "./stats/DefaultStatsFactoryPlugin";

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

	toJson(opts?: StatsValue, forToString?: boolean) {
		const options = this.compilation.createStatsOptions(opts, {
			forToString
		});

		let obj: StatsCompilation = {};

		if (options.hash) {
			obj.hash = this.#inner.getHash();
		}
		if (options.version) {
			const { version, webpackVersion } = require("../package.json");
			obj.version = webpackVersion;
			obj.rspackVersion = version;
		}

		if (options.timings) {
			obj.time = this.compilation.endTime! - this.compilation.startTime!;
		}
		if (options.builtAt) {
			obj.builtAt = this.compilation.endTime;
		}
		if (options.publicPath) {
			obj.publicPath = this.compilation.outputOptions.publicPath;
		}
		if (options.outputPath) {
			obj.outputPath = this.compilation.outputOptions.path;
		}
		if (options.assets) {
			const { assets, assetsByChunkName } = this.#inner.getAssets();
			obj.assets = assets;
			obj.assetsByChunkName = assetsByChunkName.reduce<
				Record<string, string[]>
			>((acc, cur) => {
				acc[cur.name] = cur.files;
				return acc;
			}, {});
		}
		if (options.chunks) {
			obj.chunks = this.#inner.getChunks(
				options.chunkModules!,
				options.chunkRelations!,
				options.reasons!,
				options.moduleAssets!,
				options.nestedModules!,
				options.source!
			);
		}
		if (options.modules) {
			obj.modules = this.#inner.getModules(
				options.reasons!,
				options.moduleAssets!,
				options.nestedModules!,
				options.source!
			);
		}

		if (options.entrypoints) {
			obj.entrypoints = this.#inner
				.getEntrypoints()
				.reduce<Record<string, binding.JsStatsChunkGroup>>((acc, cur) => {
					acc[cur.name] = cur;
					return acc;
				}, {});
		}
		if (options.chunkGroups) {
			obj.namedChunkGroups = this.#inner
				.getNamedChunkGroups()
				.reduce<Record<string, binding.JsStatsChunkGroup>>((acc, cur) => {
					acc[cur.name] = cur;
					return acc;
				}, {});
		}
		if (options.errors) {
			obj.errors = this.#inner.getErrors();
		}
		if (options.errorsCount) {
			obj.errorsCount = (obj.errors ?? this.#inner.getErrors()).length;
		}
		if (options.warnings) {
			obj.warnings = this.#inner.getWarnings();
		}
		if (options.warningsCount) {
			obj.warningsCount = (obj.warnings ?? this.#inner.getWarnings()).length;
		}
		if (obj.modules && forToString) {
			obj.filteredModules = obj.modules.length - 15;
			obj.modules = obj.modules.slice(0, 15);
		}
		return obj;
	}

	toString(opts?: StatsValue) {
		const options = this.compilation.createStatsOptions(opts, {
			forToString: true
		});
		const obj = this.toJson(options, true);

		const statsPrinter = this.compilation.createStatsPrinter(options);

		const result = statsPrinter.print("compilation", obj);

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
