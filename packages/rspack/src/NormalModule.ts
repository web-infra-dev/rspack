import util from "util";
import { AsyncSeriesBailHook, HookMap, SyncHook } from "tapable";

import { Compilation } from "./Compilation";
import { LoaderContext } from "./config";

const compilationHooksMap = new WeakMap<
	Compilation,
	{
		loader: SyncHook<[LoaderContext]>;
		readResourceForScheme: any;
		readResource: HookMap<
			AsyncSeriesBailHook<[LoaderContext], string | Buffer>
		>;
	}
>();

const createFakeHook = <T extends Record<string, any>>(
	fakeHook: T,
	message?: string,
	code?: string
): FakeHook<T> => {
	if (message && code) {
		fakeHook = deprecateAllProperties(fakeHook, message, code);
	}
	return Object.freeze(Object.assign(fakeHook, { _fakeHook: true }));
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
// Actually it is just a NormalModule proxy, used for hooks api alignment
// Maybe we can 1:1 align to webpack NormalModule once we found a better way to reduce communicate overhead between rust and js
export class NormalModule {
	constructor() {}
	static getCompilationHooks(compilation: Compilation) {
		if (!(compilation instanceof Compilation)) {
			throw new TypeError(
				"The 'compilation' argument must be an instance of Compilation"
			);
		}
		let hooks = compilationHooksMap.get(compilation);
		if (hooks === undefined) {
			hooks = {
				loader: new SyncHook(["loaderContext"]),
				// beforeLoaders: new SyncHook(["loaders", "module", "loaderContext"]),
				// beforeParse: new SyncHook(["module"]),
				// beforeSnapshot: new SyncHook(["module"]),
				// TODO webpack 6 deprecate
				readResourceForScheme: new HookMap(scheme => {
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
					});
				}),
				readResource: new HookMap(
					() => new AsyncSeriesBailHook(["loaderContext"])
				)
				// needBuild: new AsyncSeriesBailHook(["module", "context"])
			};
			compilationHooksMap.set(compilation, hooks);
		}
		return hooks;
	}
}
