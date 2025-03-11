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

const define = (...args: unknown[]) => {
	const factory = args.pop() as () => {};
	factory();
};

export class CommonJsRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
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
			},
			define
		};
	}

	protected createRunner() {
		this.requirers.set("miss", this.createMissRequirer());
		this.requirers.set("entry", this.createCjsRequirer());
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

	protected createCjsRequirer(): TRunnerRequirer {
		const requireCache = Object.create(null);

		return (currentDirectory, modulePath, context = {}) => {
			if (modulePath === "@rspack/test-tools") {
				return require("@rspack/test-tools");
			}
			const file = context.file || this.getFile(modulePath, currentDirectory);
			if (!file) {
				return this.requirers.get("miss")!(currentDirectory, modulePath);
			}

			if (file.path in requireCache) {
				return requireCache[file.path].exports;
			}

			const m = {
				exports: {}
			};
			requireCache[file.path] = m;
			const currentModuleScope = this.createModuleScope(
				this.getRequire(),
				m,
				file
			);

			if (this._options.testConfig.moduleScope) {
				this._options.testConfig.moduleScope(currentModuleScope);
			}

			if (!this._options.runInNewContext) {
				file.content = `Object.assign(global, _globalAssign);\n ${file.content}`;
			}
			if (file.content.includes("__STATS__") && this._options.stats) {
				currentModuleScope.__STATS__ = this._options.stats();
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
