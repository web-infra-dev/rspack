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
import { LogType } from "./logging/Logger";

export type StatsCompilation = {
	version?: string;
	rspackVersion?: string;
	name?: string;
	hash?: string;
	time?: number;
	builtAt?: number;
	publicPath?: string;
	outputPath?: string;
	assets?: binding.JsStatsAsset[];
	assetsByChunkName?: Record<string, string[]>;
	chunks?: binding.JsStatsChunk[];
	modules?: binding.JsStatsModule[];
	entrypoints?: Record<string, binding.JsStatsChunkGroup>;
	namedChunkGroups?: Record<string, binding.JsStatsChunkGroup>;
	errors?: binding.JsStatsError[];
	errorsCount?: number;
	warnings?: binding.JsStatsWarning[];
	warningsCount?: number;
	filteredModules?: number;
	children?: StatsCompilation[];
};

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
				options.nestedModules!
			);
		}
		if (options.modules) {
			obj.modules = this.#inner.getModules(
				options.reasons!,
				options.moduleAssets!,
				options.nestedModules!
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
		const useColors = optionsOrFallback(options.colors, false);
		const obj: any = this.toJson(options, true);
		return Stats.jsonToString(obj, useColors);
	}

	static jsonToString(obj: any, useColors: boolean): any {
		const buf = [];

		const defaultColors = {
			bold: "\u001b[1m",
			yellow: "\u001b[1m\u001b[33m",
			red: "\u001b[1m\u001b[31m",
			green: "\u001b[1m\u001b[32m",
			cyan: "\u001b[1m\u001b[36m",
			magenta: "\u001b[1m\u001b[35m"
		};

		const colors: any = Object.keys(defaultColors).reduce(
			(obj, color) => {
				// @ts-expect-error
				obj[color] = str => {
					if (useColors) {
						buf.push(
							useColors === true || useColors[color] === undefined
								? // @ts-expect-error
								  defaultColors[color]
								: useColors[color]
						);
					}
					buf.push(str);
					if (useColors) {
						buf.push("\u001b[39m\u001b[22m");
					}
				};
				return obj;
			},
			{
				// @ts-expect-error
				normal: str => buf.push(str)
			}
		);

		const coloredTime = (time: number) => {
			let times = [800, 400, 200, 100];
			if (obj.time) {
				times = [obj.time / 2, obj.time / 4, obj.time / 8, obj.time / 16];
			}
			if (time < times[3]) colors.normal(`${time}ms`);
			else if (time < times[2]) colors.bold(`${time}ms`);
			else if (time < times[1]) colors.green(`${time}ms`);
			else if (time < times[0]) colors.yellow(`${time}ms`);
			else colors.red(`${time}ms`);
		};

		const newline = () => buf.push("\n");

		const getText = (
			arr: { [x: string]: { [x: string]: { value: any } } },
			row: number,
			col: number
		) => {
			return arr[row][col].value;
		};

		const table = (
			array: string | any[],
			align: string | string[],
			splitter?: string
		) => {
			const rows = array.length;
			const cols = array[0].length;
			const colSizes = new Array(cols);
			for (let col = 0; col < cols; col++) {
				colSizes[col] = 0;
			}
			for (let row = 0; row < rows; row++) {
				for (let col = 0; col < cols; col++) {
					// @ts-expect-error
					const value = `${getText(array, row, col)}`;
					if (value.length > colSizes[col]) {
						colSizes[col] = value.length;
					}
				}
			}
			for (let row = 0; row < rows; row++) {
				for (let col = 0; col < cols; col++) {
					const format = array[row][col].color;
					// @ts-expect-error
					const value = `${getText(array, row, col)}`;
					let l = value.length;
					if (align[col] === "l") {
						format(value);
					}
					for (; l < colSizes[col] && col !== cols - 1; l++) {
						colors.normal(" ");
					}
					if (align[col] === "r") {
						format(value);
					}
					if (col + 1 < cols && colSizes[col] !== 0) {
						colors.normal(splitter || "  ");
					}
				}
				newline();
			}
		};

		const getAssetColor = (
			asset: { isOverSizeLimit: any },
			defaultColor: any
		) => {
			if (asset.isOverSizeLimit) {
				return colors.yellow;
			}

			return defaultColor;
		};

		if (obj.hash) {
			colors.normal("Hash: ");
			colors.bold(obj.hash);
			newline();
		}
		if (obj.version) {
			colors.normal("Version: rspack ");
			colors.bold(obj.version);
			newline();
		}
		if (typeof obj.time === "number") {
			colors.normal("Time: ");
			colors.bold(obj.time);
			colors.normal("ms");
			newline();
		}
		if (typeof obj.builtAt === "number") {
			const builtAtDate = new Date(obj.builtAt);
			let timeZone = undefined;

			try {
				builtAtDate.toLocaleTimeString();
			} catch (err) {
				// Force UTC if runtime timezone is unsupported
				timeZone = "UTC";
			}

			colors.normal("Built at: ");
			colors.normal(
				builtAtDate.toLocaleDateString(undefined, {
					day: "2-digit",
					month: "2-digit",
					year: "numeric",
					timeZone
				})
			);
			colors.normal(" ");
			colors.bold(builtAtDate.toLocaleTimeString(undefined, { timeZone }));
			newline();
		}
		if (obj.env) {
			colors.normal("Environment (--env): ");
			colors.bold(JSON.stringify(obj.env, null, 2));
			newline();
		}
		if (obj.publicPath) {
			colors.normal("PublicPath: ");
			colors.bold(obj.publicPath);
			newline();
		}

		if (obj.assets && obj.assets.length > 0) {
			const t = [
				[
					{
						value: "Asset",
						color: colors.bold
					},
					{
						value: "Size",
						color: colors.bold
					},
					{
						value: "Chunks",
						color: colors.bold
					},
					{
						value: "",
						color: colors.bold
					},
					{
						value: "",
						color: colors.bold
					},
					{
						value: "Chunk Names",
						color: colors.bold
					}
				]
			];
			for (const asset of obj.assets) {
				t.push([
					{
						value: asset.name,
						color: getAssetColor(asset, colors.green)
					},
					{
						value: SizeFormatHelpers.formatSize(asset.size),
						color: getAssetColor(asset, colors.normal)
					},
					{
						value: asset.chunks.join(", "),
						color: colors.bold
					},
					{
						value: [
							asset.emitted && "[emitted]",
							asset.info?.immutable && "[immutable]",
							asset.info?.development && "[dev]",
							asset.info?.hotModuleReplacement && "[hmr]"
						]
							.filter(Boolean)
							.join(" "),
						color: colors.green
					},
					{
						value: asset.isOverSizeLimit ? "[big]" : "",
						color: getAssetColor(asset, colors.normal)
					},
					{
						value: asset.chunkNames.join(", "),
						color: colors.normal
					}
				]);
			}
			table(t, "rrrlll");
		}
		if (obj.filteredAssets > 0) {
			colors.normal(" ");
			if (obj.assets.length > 0) colors.normal("+ ");
			colors.normal(obj.filteredAssets);
			if (obj.assets.length > 0) colors.normal(" hidden");
			colors.normal(obj.filteredAssets !== 1 ? " assets" : " asset");
			newline();
		}

		const processChunkGroups = (
			namedGroups: { [x: string]: any },
			prefix: string
		) => {
			for (const name of Object.keys(namedGroups)) {
				const cg = namedGroups[name];
				colors.normal(`${prefix} `);
				colors.bold(name);
				if (cg.isOverSizeLimit) {
					colors.normal(" ");
					colors.yellow("[big]");
				}
				colors.normal(" =");
				for (const asset of cg.assets) {
					colors.normal(" ");
					colors.green(asset.name);
				}
				// for (const name of Object.keys(cg.childAssets)) {
				// 	const assets = cg.childAssets[name];
				// 	if (assets && assets.length > 0) {
				// 		colors.normal(" ");
				// 		colors.magenta(`(${name}:`);
				// 		for (const asset of assets) {
				// 			colors.normal(" ");
				// 			colors.green(asset);
				// 		}
				// 		colors.magenta(")");
				// 	}
				// }
				newline();
			}
		};

		if (obj.entrypoints) {
			processChunkGroups(obj.entrypoints, "Entrypoint");
		}

		if (obj.namedChunkGroups) {
			let outputChunkGroups = obj.namedChunkGroups;
			if (obj.entrypoints) {
				outputChunkGroups = Object.keys(outputChunkGroups)
					.filter(name => !obj.entrypoints[name])
					.reduce((result, name) => {
						// @ts-expect-error
						result[name] = obj.namedChunkGroups[name];
						return result;
					}, {});
			}
			processChunkGroups(outputChunkGroups, "Chunk Group");
		}

		const modulesByIdentifier = {};
		if (obj.modules) {
			for (const module of obj.modules) {
				// @ts-expect-error
				modulesByIdentifier[`$${module.identifier}`] = module;
			}
		} else if (obj.chunks) {
			for (const chunk of obj.chunks) {
				if (chunk.modules) {
					for (const module of chunk.modules) {
						// @ts-expect-error
						modulesByIdentifier[`$${module.identifier}`] = module;
					}
				}
			}
		}

		const processModuleAttributes = (module: {
			size: any;
			chunks: any;
			depth: any;
			cacheable: boolean;
			optional: any;
			built: any;
			assets: string | any[];
			prefetched: any;
			failed: any;
			warnings: number;
			errors: number;
		}) => {
			colors.normal(" ");
			colors.normal(SizeFormatHelpers.formatSize(module.size));
			if (module.chunks) {
				for (const chunk of module.chunks) {
					colors.normal(" {");
					colors.yellow(chunk);
					colors.normal("}");
				}
			}
			if (typeof module.depth === "number") {
				colors.normal(` [depth ${module.depth}]`);
			}
			if (module.cacheable === false) {
				colors.red(" [not cacheable]");
			}
			if (module.optional) {
				colors.yellow(" [optional]");
			}
			if (module.built) {
				colors.green(" [built]");
			}
			if (module.assets && module.assets.length) {
				colors.magenta(
					` [${module.assets.length} asset${
						module.assets.length === 1 ? "" : "s"
					}]`
				);
			}
			if (module.prefetched) {
				colors.magenta(" [prefetched]");
			}
			if (module.failed) colors.red(" [failed]");
			if (module.warnings) {
				colors.yellow(
					` [${module.warnings} warning${module.warnings === 1 ? "" : "s"}]`
				);
			}
			if (module.errors) {
				colors.red(
					` [${module.errors} error${module.errors === 1 ? "" : "s"}]`
				);
			}
		};

		const processModuleContent = (
			module: {
				providedExports: any[];
				usedExports: boolean | any[] | null | undefined;
				optimizationBailout: any;
				reasons: any;
				profile: { [x: string]: any };
				issuerPath: any;
				modules: any;
			},
			prefix: string
		) => {
			if (Array.isArray(module.providedExports)) {
				colors.normal(prefix);
				if (module.providedExports.length === 0) {
					colors.cyan("[no exports]");
				} else {
					colors.cyan(`[exports: ${module.providedExports.join(", ")}]`);
				}
				newline();
			}
			if (module.usedExports !== undefined) {
				if (module.usedExports !== true) {
					colors.normal(prefix);
					if (module.usedExports === null) {
						colors.cyan("[used exports unknown]");
					} else if (module.usedExports === false) {
						colors.cyan("[no exports used]");
					} else if (
						Array.isArray(module.usedExports) &&
						module.usedExports.length === 0
					) {
						colors.cyan("[no exports used]");
					} else if (Array.isArray(module.usedExports)) {
						const providedExportsCount = Array.isArray(module.providedExports)
							? module.providedExports.length
							: null;
						if (
							providedExportsCount !== null &&
							providedExportsCount === module.usedExports.length
						) {
							colors.cyan("[all exports used]");
						} else {
							colors.cyan(
								`[only some exports used: ${module.usedExports.join(", ")}]`
							);
						}
					}
					newline();
				}
			}
			if (Array.isArray(module.optimizationBailout)) {
				for (const item of module.optimizationBailout) {
					colors.normal(prefix);
					colors.yellow(item);
					newline();
				}
			}
			if (module.reasons) {
				for (const reason of module.reasons) {
					colors.normal(prefix);
					if (reason.type) {
						colors.normal(reason.type);
						colors.normal(" ");
					}
					if (reason.userRequest) {
						colors.cyan(reason.userRequest);
						colors.normal(" ");
					}
					if (reason.moduleId) {
						colors.normal("[");
						colors.normal(reason.moduleId);
						colors.normal("]");
					}
					if (reason.module && reason.module !== reason.moduleId) {
						colors.normal(" ");
						colors.magenta(reason.module);
					}
					if (reason.loc) {
						colors.normal(" ");
						colors.normal(reason.loc);
					}
					if (reason.explanation) {
						colors.normal(" ");
						colors.cyan(reason.explanation);
					}
					newline();
				}
			}
			if (module.profile) {
				colors.normal(prefix);
				let sum = 0;
				if (module.issuerPath) {
					for (const m of module.issuerPath) {
						colors.normal("[");
						colors.normal(m.id);
						colors.normal("] ");
						if (m.profile) {
							const time = (m.profile.factory || 0) + (m.profile.building || 0);
							coloredTime(time);
							sum += time;
							colors.normal(" ");
						}
						colors.normal("-> ");
					}
				}
				for (const key of Object.keys(module.profile)) {
					colors.normal(`${key}:`);
					const time = module.profile[key];
					coloredTime(time);
					colors.normal(" ");
					sum += time;
				}
				colors.normal("= ");
				coloredTime(sum);
				newline();
			}
			if (module.modules) {
				// @ts-expect-error
				processModulesList(module, prefix + "| ");
			}
		};

		const processModulesList = (
			obj: { modules: string | any[]; filteredModules: number },
			prefix: string
		) => {
			if (obj.modules) {
				let maxModuleId = 0;
				for (const module of obj.modules) {
					if (typeof module.id === "number") {
						if (maxModuleId < module.id) maxModuleId = module.id;
					}
				}
				let contentPrefix = prefix + "    ";
				if (maxModuleId >= 10) contentPrefix += " ";
				if (maxModuleId >= 100) contentPrefix += " ";
				if (maxModuleId >= 1000) contentPrefix += " ";
				for (const module of obj.modules) {
					colors.normal(prefix);
					const name = module.name || module.identifier;
					if (typeof module.id === "string" || typeof module.id === "number") {
						if (typeof module.id === "number") {
							if (module.id < 1000 && maxModuleId >= 1000) colors.normal(" ");
							if (module.id < 100 && maxModuleId >= 100) colors.normal(" ");
							if (module.id < 10 && maxModuleId >= 10) colors.normal(" ");
						} else {
							if (maxModuleId >= 1000) colors.normal(" ");
							if (maxModuleId >= 100) colors.normal(" ");
							if (maxModuleId >= 10) colors.normal(" ");
						}
						if (name !== module.id) {
							colors.normal("[");
							colors.normal(module.id);
							colors.normal("]");
							colors.normal(" ");
						} else {
							colors.normal("[");
							colors.bold(module.id);
							colors.normal("]");
						}
					}
					if (name !== module.id) {
						colors.bold(name);
					}
					processModuleAttributes(module);
					newline();
					processModuleContent(module, contentPrefix);
				}
				if (obj.filteredModules > 0) {
					colors.normal(prefix);
					colors.normal("   ");
					if (obj.modules.length > 0) colors.normal(" + ");
					colors.normal(obj.filteredModules);
					if (obj.modules.length > 0) colors.normal(" hidden");
					colors.normal(obj.filteredModules !== 1 ? " modules" : " module");
					newline();
				}
			}
		};

		if (obj.chunks) {
			for (const chunk of obj.chunks) {
				colors.normal("chunk ");
				if (chunk.id < 1000) colors.normal(" ");
				if (chunk.id < 100) colors.normal(" ");
				if (chunk.id < 10) colors.normal(" ");
				colors.normal("{");
				colors.yellow(chunk.id);
				colors.normal("} ");
				colors.green(chunk.files.join(", "));
				if (chunk.names && chunk.names.length > 0) {
					colors.normal(" (");
					colors.normal(chunk.names.join(", "));
					colors.normal(")");
				}
				colors.normal(" ");
				colors.normal(SizeFormatHelpers.formatSize(chunk.size));
				// TODO: stats chunk relation
				// for (const id of chunk.parents) {
				// 	colors.normal(" <{");
				// 	colors.yellow(id);
				// 	colors.normal("}>");
				// }
				// for (const id of chunk.siblings) {
				// 	colors.normal(" ={");
				// 	colors.yellow(id);
				// 	colors.normal("}=");
				// }
				// for (const id of chunk.children) {
				// 	colors.normal(" >{");
				// 	colors.yellow(id);
				// 	colors.normal("}<");
				// }
				if (chunk.childrenByOrder) {
					for (const name of Object.keys(chunk.childrenByOrder)) {
						const children = chunk.childrenByOrder[name];
						colors.normal(" ");
						colors.magenta(`(${name}:`);
						for (const id of children) {
							colors.normal(" {");
							colors.yellow(id);
							colors.normal("}");
						}
						colors.magenta(")");
					}
				}
				if (chunk.entry) {
					colors.yellow(" [entry]");
				} else if (chunk.initial) {
					colors.yellow(" [initial]");
				}
				if (chunk.rendered) {
					colors.green(" [rendered]");
				}
				if (chunk.recorded) {
					colors.green(" [recorded]");
				}
				if (chunk.reason) {
					colors.yellow(` ${chunk.reason}`);
				}
				newline();
				if (chunk.origins) {
					for (const origin of chunk.origins) {
						colors.normal("    > ");
						if (origin.reasons && origin.reasons.length) {
							colors.yellow(origin.reasons.join(" "));
							colors.normal(" ");
						}
						if (origin.request) {
							colors.normal(origin.request);
							colors.normal(" ");
						}
						if (origin.module) {
							colors.normal("[");
							colors.normal(origin.moduleId);
							colors.normal("] ");
							// @ts-expect-error
							const module = modulesByIdentifier[`$${origin.module}`];
							if (module) {
								colors.bold(module.name);
								colors.normal(" ");
							}
						}
						if (origin.loc) {
							colors.normal(origin.loc);
						}
						newline();
					}
				}
				processModulesList(chunk, " ");
			}
		}

		processModulesList(obj, "");

		if (obj.logging) {
			for (const origin of Object.keys(obj.logging)) {
				const logData = obj.logging[origin];
				if (logData.entries.length > 0) {
					newline();
					if (logData.debug) {
						colors.red("DEBUG ");
					}
					colors.bold("LOG from " + origin);
					newline();
					let indent = "";
					for (const entry of logData.entries) {
						let color = colors.normal;
						let prefix = "    ";
						switch (entry.type) {
							case LogType.clear:
								colors.normal(`${indent}-------`);
								newline();
								continue;
							case LogType.error:
								color = colors.red;
								prefix = "<e> ";
								break;
							case LogType.warn:
								color = colors.yellow;
								prefix = "<w> ";
								break;
							case LogType.info:
								color = colors.green;
								prefix = "<i> ";
								break;
							case LogType.log:
								color = colors.bold;
								break;
							case LogType.trace:
							case LogType.debug:
								color = colors.normal;
								break;
							case LogType.status:
								color = colors.magenta;
								prefix = "<s> ";
								break;
							case LogType.profile:
								color = colors.magenta;
								prefix = "<p> ";
								break;
							case LogType.profileEnd:
								color = colors.magenta;
								prefix = "</p> ";
								break;
							case LogType.time:
								color = colors.magenta;
								prefix = "<t> ";
								break;
							case LogType.group:
								color = colors.cyan;
								prefix = "<-> ";
								break;
							case LogType.groupCollapsed:
								color = colors.cyan;
								prefix = "<+> ";
								break;
							case LogType.groupEnd:
								if (indent.length >= 2)
									indent = indent.slice(0, indent.length - 2);
								continue;
						}
						if (entry.message) {
							for (const line of entry.message.split("\n")) {
								colors.normal(`${indent}${prefix}`);
								color(line);
								newline();
							}
						}
						if (entry.trace) {
							for (const line of entry.trace) {
								colors.normal(`${indent}| ${line}`);
								newline();
							}
						}
						switch (entry.type) {
							case LogType.group:
								indent += "  ";
								break;
						}
					}
					if (logData.filteredEntries) {
						colors.normal(`+ ${logData.filteredEntries} hidden lines`);
						newline();
					}
				}
			}
		}

		if (obj.warnings) {
			for (const warning of obj.warnings) {
				newline();
				// formatted warning already have color.
				colors.normal(formatError(warning));
				newline();
			}
		}
		if (obj.errors) {
			for (const error of obj.errors) {
				newline();
				// formatted error already have color.
				colors.normal(formatError(error));
				newline();
			}
		}
		if (obj.children) {
			for (const child of obj.children) {
				const childString = Stats.jsonToString(child, useColors);
				if (childString) {
					if (child.name) {
						colors.normal("Child ");
						colors.bold(child.name);
						colors.normal(":");
					} else {
						colors.normal("Child");
					}
					newline();
					buf.push("    ");
					buf.push(childString.replace(/\n/g, "\n    "));
					newline();
				}
			}
		}
		if (obj.needAdditionalPass) {
			colors.yellow(
				"Compilation needs an additional pass and will compile again."
			);
		}

		while (buf[buf.length - 1] === "\n") {
			buf.pop();
		}
		return buf.join("");
	}
}

const SizeFormatHelpers = {
	formatSize: (size: unknown) => {
		if (typeof size !== "number" || Number.isNaN(size) === true) {
			return "unknown size";
		}

		if (size <= 0) {
			return "0 bytes";
		}

		const abbreviations = ["bytes", "KiB", "MiB", "GiB"];
		const index = Math.floor(Math.log(size) / Math.log(1024));

		return `${+(size / Math.pow(1024, index)).toPrecision(3)} ${
			abbreviations[index]
		}`;
	}
};

const formatError = (e: binding.JsStatsError) => {
	return e.formatted;
};

export const optionsOrFallback = (
	options: boolean | undefined,
	fallback: boolean
) => options ?? fallback;

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
