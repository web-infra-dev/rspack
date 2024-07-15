import path from "path";
import vm from "vm";

import type { ECompilerType } from "../../type";
import type {
	IBasicGlobalContext,
	IBasicModuleScope,
	TBasicRunnerFile,
	TModuleObject,
	TRunnerRequirer
} from "../type";
import { BasicRunner } from "./basic";

const define = (...args: unknown[]) => {
	const factory = args.pop() as () => {};
	factory();
};

export class CommonJsRunner<
	T extends ECompilerType = ECompilerType.Rspack
> extends BasicRunner<T> {
	protected createGlobalContext(): IBasicGlobalContext {
		return {
			console: console,
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
			...this._options.env
		};
		if (this._options.stats) {
			baseModuleScope["__STATS__"] = this._options.stats;
		}
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
			} else {
				return require(
					modulePathStr.startsWith("node:")
						? modulePathStr.slice(5)
						: modulePathStr
				);
			}
		};
	}

	protected createCjsRequirer(): TRunnerRequirer {
		const requireCache = Object.create(null);

		return (currentDirectory, modulePath, context = {}) => {
			const file =
				context["file"] || this.getFile(modulePath, currentDirectory);
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
