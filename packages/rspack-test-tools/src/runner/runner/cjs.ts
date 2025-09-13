import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";

import type { ECompilerType } from "../../type";
import type {
	IBasicGlobalContext,
	IBasicModuleScope,
	TBasicRunnerFile,
	TModuleObject,
	TRunnerRequirer
} from "../type";
import { BasicRunner } from "./basic";

declare global {
	var printLogger: boolean;
}

export class CommonJsRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	protected requireCache = Object.create(null);
	protected createGlobalContext(): IBasicGlobalContext {
		return {
			console: {
				log: (...args: any[]) => {
					if (printLogger) {
						console.log(...args);
					}
				},
				warn: (...args: any[]) => {
					if (printLogger) {
						console.warn(...args);
					}
				},
				error: (...args: any[]) => {
					console.error(...args);
				},
				info: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				debug: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				trace: (...args: any[]) => {
					if (printLogger) {
						console.info(...args);
					}
				},
				assert: (...args: any[]) => {
					console.assert(...args);
				},
				clear: () => {
					console.clear();
				}
			},
			setTimeout: ((
				cb: (...args: any[]) => void,
				ms: number | undefined,
				...args: any
			) => {
				const timeout = setTimeout(cb, ms, ...args);
				timeout.unref();
				return timeout;
			}) as typeof setTimeout,
			clearTimeout: clearTimeout
		};
	}

	protected createBaseModuleScope(): IBasicModuleScope {
		const baseModuleScope: IBasicModuleScope = {
			console: this.globalContext!.console,
			setTimeout: this.globalContext!.setTimeout,
			clearTimeout: this.globalContext!.clearTimeout,
			nsObj: (m: Object) => {
				Object.defineProperty(m, Symbol.toStringTag, {
					value: "Module"
				});
				return m;
			},
			process,
			URL,
			Blob,
			Symbol,
			__MODE__: this._options.compilerOptions.mode,
			__SNAPSHOT__: path.join(this._options.source, "__snapshot__"),
			...this._options.env
		};
		return baseModuleScope;
	}

	protected createModuleScope(
		requireFn: TRunnerRequirer,
		m: TModuleObject,
		file: TBasicRunnerFile
	): IBasicModuleScope {
		return {
			...this.baseModuleScope!,
			require: requireFn.bind(null, path.dirname(file.path)),
			module: m,
			exports: m.exports,
			__dirname: path.dirname(file.path),
			__filename: file.path,
			_globalAssign: {
				expect: this._options.env.expect
			}
		};
	}

	protected createRunner() {
		this.requirers.set("miss", this.createMissRequirer());
		this.requirers.set("entry", this.createCjsRequirer());
		this.requirers.set("json", this.createJsonRequirer());
	}

	protected createMissRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			const modulePathStr = modulePath as string;
			const modules = this._options.testConfig.modules;
			if (modules && modulePathStr in modules) {
				return modules[modulePathStr];
			}
			return require(
				modulePathStr.startsWith("node:")
					? modulePathStr.slice(5)
					: modulePathStr
			);
		};
	}

	protected createJsonRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			if (Array.isArray(modulePath)) {
				throw new Error("Array module path is not supported in hot cases");
			}
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}
			return JSON.parse(
				fs.readFileSync(path.join(this._options.dist, modulePath), "utf-8")
			);
		};
	}

	protected createCjsRequirer(): TRunnerRequirer {
		return (currentDirectory, modulePath, context = {}) => {
			if (modulePath === "@rspack/test-tools") {
				return require("@rspack/test-tools");
			}
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path.endsWith(".json")) {
				return this.requirers.get("json")!(
					currentDirectory,
					modulePath,
					context
				);
			}

			if (file.path in this.requireCache) {
				return this.requireCache[file.path].exports;
			}

			const m = {
				exports: {}
			};
			this.requireCache[file.path] = m;
			const currentModuleScope = this.createModuleScope(
				this.getRequire(),
				m,
				file
			);

			if (this._options.testConfig.moduleScope) {
				this._options.testConfig.moduleScope(
					currentModuleScope,
					this._options.stats,
					this._options.compilerOptions
				);
			}

			if (!this._options.runInNewContext) {
				file.content = `Object.assign(global, _globalAssign);\n ${file.content}`;
			}
			if (file.content.includes("__STATS__") && this._options.stats) {
				currentModuleScope.__STATS__ = this._options.stats();
			}
			if (file.content.includes("__STATS_I__")) {
				const statsIndex = this._options.stats?.()?.__index__;
				if (typeof statsIndex === "number") {
					currentModuleScope.__STATS_I__ = statsIndex;
				}
			}

			const args = Object.keys(currentModuleScope);
			const argValues = args.map(arg => currentModuleScope[arg]);
			const code = `(function(${args.join(", ")}) {
        ${file.content}
      })`;

			this.preExecute(code, file);
			const fn = this._options.runInNewContext
				? vm.runInNewContext(code, this.globalContext!, file.path)
				: vm.runInThisContext(code, file.path);
			fn.call(
				this._options.testConfig.nonEsmThis
					? this._options.testConfig.nonEsmThis(modulePath)
					: m.exports,
				...argValues
			);

			this.postExecute(m, file);
			return m.exports;
		};
	}
}
