/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/MultiStats.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { KnownCreateStatsOptionsContext } from "./Compilation";
import { Stats } from "./Stats";
import { StatsCompilation } from "./stats/statsFactoryUtils";
import { indent } from "./util";
import * as identifierUtils from "./util/identifier";

export default class MultiStats {
	stats: Stats[];

	constructor(stats: Stats[]) {
		this.stats = stats;
	}

	get hash(): string {
		return this.stats.map(stat => stat.hash).join("");
	}

	hasErrors(): boolean {
		return this.stats.some(stat => stat.hasErrors());
	}

	hasWarnings(): boolean {
		return this.stats.some(stat => stat.hasWarnings());
	}

	#createChildOptions(
		options: { [x: string]: any; children?: any },
		context: (KnownCreateStatsOptionsContext & Record<string, any>) | undefined
	) {
		if (!options) {
			options = {};
		}
		const { children: childrenOptions = undefined, ...baseOptions } =
			typeof options === "string" ? { preset: options } : options;

		const children = this.stats.map((stat, idx) => {
			const childOptions = Array.isArray(childrenOptions)
				? childrenOptions[idx]
				: childrenOptions;
			return stat.compilation.createStatsOptions(
				{
					...baseOptions,
					...(typeof childOptions === "string"
						? { preset: childOptions }
						: childOptions && typeof childOptions === "object"
						? childOptions
						: undefined)
				},
				context
			);
		});
		return {
			hash: children.every(o => o.hash),
			errorsCount: children.every(o => o.errorsCount),
			warningsCount: children.every(o => o.warningsCount),
			errors: children.every(o => o.errors),
			warnings: children.every(o => o.warnings),
			children
		};
	}

	toJson(options?: any) {
		options = this.#createChildOptions(options, { forToString: false });

		const obj: StatsCompilation = {};
		obj.children = this.stats.map((stat, idx) => {
			const obj = stat.toJson(options.children[idx]);
			const compilationName = stat.compilation.name;
			const name =
				compilationName &&
				identifierUtils.makePathsRelative(
					options.context,
					compilationName,
					stat.compilation.compiler.root
				);
			obj.name = name;
			return obj;
		});
		if (options.version) {
			obj.rspackVersion = require("../package.json").version;
			obj.version = require("../package.json").webpackVersion;
		}
		if (options.hash) {
			obj.hash = obj.children.map(j => j.hash).join("");
		}
		const mapError = (j: any, obj: any) => {
			return {
				...obj,
				compilerPath: obj.compilerPath
					? `${j.name}.${obj.compilerPath}`
					: j.name
			};
		};
		if (options.errors) {
			obj.errors = [];
			for (const j of obj.children) {
				for (const i of j.errors || []) {
					obj.errors.push(mapError(j, i));
				}
			}
		}
		if (options.warnings) {
			obj.warnings = [];
			for (const j of obj.children) {
				for (const i of j.warnings || []) {
					obj.warnings.push(mapError(j, i));
				}
			}
		}
		if (options.errorsCount) {
			obj.errorsCount = 0;
			for (const j of obj.children) {
				obj.errorsCount += j.errorsCount || 0;
			}
		}
		if (options.warningsCount) {
			obj.warningsCount = 0;
			for (const j of obj.children) {
				obj.warningsCount += j.warningsCount || 0;
			}
		}
		return obj;
	}

	toString(options: any) {
		options = this.#createChildOptions(options, { forToString: true });
		const results = this.stats.map((stat, idx) => {
			const str = stat.toString(options.children[idx]);
			const compilationName = stat.compilation.name;
			const name =
				compilationName &&
				identifierUtils
					.makePathsRelative(
						options.context,
						compilationName,
						stat.compilation.compiler.root
					)
					.replace(/\|/g, " ");
			if (!str) return str;
			return name ? `${name}:\n${indent(str, "  ")}` : str;
		});
		return results.filter(Boolean).join("\n\n");
	}
}

export { MultiStats };
