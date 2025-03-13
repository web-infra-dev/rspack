import util from "node:util";
import * as liteTapable from "@rspack/lite-tapable";

import { Compilation } from "./Compilation";
import type { Module } from "./Module";
import type { LoaderContext } from "./config";

import * as binding from "@rspack/binding";
import type { Source } from "webpack-sources";
import { DependenciesBlock } from "./DependenciesBlock";
import { JsSource } from "./util/source";

Object.defineProperty(binding.NormalModule.prototype, "blocks", {
	enumerable: true,
	configurable: true,
	get(this: binding.NormalModule) {
		return this._blocks.map(block => DependenciesBlock.__from_binding(block));
	}
});
Object.defineProperty(binding.NormalModule.prototype, "originalSource", {
	enumerable: true,
	configurable: true,
	value(this: binding.NormalModule) {
		const originalSource = this._originalSource();
		if (originalSource) {
			return JsSource.__from_binding(originalSource);
		}
		return null;
	}
});
Object.defineProperty(binding.NormalModule.prototype, "emitFile", {
	enumerable: true,
	configurable: true,
	value(
		this: binding.NormalModule,
		filename: string,
		source: Source,
		assetInfo?: binding.AssetInfo
	) {
		return this._emitFile(filename, JsSource.__to_binding(source), assetInfo);
	}
});

interface NormalModuleCompilationHooks {
	loader: liteTapable.SyncHook<[LoaderContext, Module]>;
	readResourceForScheme: any;
	readResource: liteTapable.HookMap<
		liteTapable.AsyncSeriesBailHook<[LoaderContext], string | Buffer>
	>;
}

const compilationHooksMap = new WeakMap<
	Compilation,
	NormalModuleCompilationHooks
>();

const createFakeHook = <T extends Record<string, any>>(
	fakeHook: T,
	message?: string,
	code?: string
): FakeHook<T> => {
	return Object.freeze(
		Object.assign(
			message && code
				? deprecateAllProperties(fakeHook, message, code)
				: fakeHook,
			{ _fakeHook: true }
		)
	);
};
type FakeHook<T> = {
	_fakeHook: true;
} & T;
const deprecateAllProperties = <O extends object>(
	obj: O,
	message: string,
	code: string
) => {
	const newObj: any = {};
	const descriptors = Object.getOwnPropertyDescriptors(obj);
	for (const name of Object.keys(descriptors)) {
		const descriptor = descriptors[name];
		if (typeof descriptor.value === "function") {
			Object.defineProperty(newObj, name, {
				...descriptor,
				value: util.deprecate(descriptor.value, message, code)
			});
		} else if (descriptor.get || descriptor.set) {
			Object.defineProperty(newObj, name, {
				...descriptor,
				get: descriptor.get && util.deprecate(descriptor.get, message, code),
				set: descriptor.set && util.deprecate(descriptor.set, message, code)
			});
		} else {
			let value = descriptor.value;
			Object.defineProperty(newObj, name, {
				configurable: descriptor.configurable,
				enumerable: descriptor.enumerable,
				get: util.deprecate(() => value, message, code),
				set: descriptor.writable
					? util.deprecate((v: any) => (value = v), message, code)
					: undefined
			});
		}
	}
	return newObj;
};

Object.defineProperty(binding.NormalModule, "getCompilationHooks", {
	enumerable: true,
	configurable: true,
	value(compilation: Compilation): NormalModuleCompilationHooks {
		if (!(compilation instanceof Compilation)) {
			throw new TypeError(
				"The 'compilation' argument must be an instance of Compilation"
			);
		}
		let hooks = compilationHooksMap.get(compilation);
		if (hooks === undefined) {
			hooks = {
				loader: new liteTapable.SyncHook(["loaderContext", "module"]),
				// TODO webpack 6 deprecate
				readResourceForScheme: new liteTapable.HookMap(scheme => {
					const hook = hooks!.readResource.for(scheme);
					return createFakeHook({
						tap: (options: string, fn: any) =>
							hook.tap(options, (loaderContext: LoaderContext) =>
								fn(loaderContext.resource)
							),
						tapAsync: (options: string, fn: any) =>
							hook.tapAsync(
								options,
								(loaderContext: LoaderContext, callback: any) =>
									fn(loaderContext.resource, callback)
							),
						tapPromise: (options: string, fn: any) =>
							hook.tapPromise(options, (loaderContext: LoaderContext) =>
								fn(loaderContext.resource)
							)
					}) as any;
				}),
				readResource: new liteTapable.HookMap(
					() => new liteTapable.AsyncSeriesBailHook(["loaderContext"])
				)
			};
			compilationHooksMap.set(compilation, hooks);
		}
		return hooks;
	}
});

declare module "@rspack/binding" {
	interface NormalModuleConstructor {
		getCompilationHooks(compilation: Compilation): NormalModuleCompilationHooks;
	}
}

export { NormalModule } from "@rspack/binding";
