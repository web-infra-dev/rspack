import { JavaScriptTracer } from ".";
// adjust from webpack's ProfilingPlugin https://github.com/webpack/webpack/blob/dec18718be5dfba28f067fb3827dd620a1f33667/lib/debug/ProfilingPlugin.js#L1
import type { Compiler } from "../exports";
const PLUGIN_NAME = "TraceHookPlugin";
type FullTap = Tap & {
	type: "sync" | "async" | "promise";
	fn: Function;
};
type Tap = TapOptions & {
	name: string;
};
type TapOptions = {
	before?: string;
	stage?: number;
};
// This plugin is used to trace the execution of various hooks in the build process.
const makeInterceptorFor =
	(compilerName: string, tracer: typeof JavaScriptTracer) =>
	(hookName: string) => ({
		register: (tapInfo: FullTap) => {
			const { name, type, fn: internalFn } = tapInfo;
			const newFn =
				// Don't tap our own hooks to ensure stream can close cleanly
				name === PLUGIN_NAME
					? internalFn
					: makeNewTraceTapFn(compilerName, hookName, tracer, {
							name,
							type,
							fn: internalFn
						});
			return { ...tapInfo, fn: newFn };
		}
	});
const interceptAllHooksFor = (
	instance: any,
	tracer: typeof JavaScriptTracer,
	logLabel: string
) => {
	if (Reflect.has(instance, "hooks")) {
		for (const hookName of Object.keys(instance.hooks)) {
			const hook = instance.hooks[hookName];
			if (hook && !hook._fakeHook) {
				hook.intercept(makeInterceptorFor(logLabel, tracer)(hookName));
			}
		}
	}
};
const makeNewTraceTapFn = (
	compilerName: string,
	hookName: string,
	tracer: typeof JavaScriptTracer,
	{ name: pluginName, type, fn }: { name: string; type: string; fn: Function }
) => {
	const name = `${pluginName}:${hookName}`;
	const id = pluginName; // used for track
	switch (type) {
		case "promise":
			return (...args: any[]) => {
				const id2 = tracer.counter++;
				tracer.startAsync({
					name,
					id,
					args: {
						id2,
						compilerName,
						hookName,
						pluginName
					}
				});
				const promise =
					/** @type {Promise<(...args: EXPECTED_ANY[]) => EXPECTED_ANY>} */
					fn(...args);
				return promise.then((r: any) => {
					tracer.endAsync({
						name,
						id,
						args: {
							id2,
							compilerName,
							hookName,
							pluginName
						}
					});
					return r;
				});
			};
		case "async":
			return (...args: any[]) => {
				const id2 = tracer.counter++;
				tracer.startAsync({
					name,
					id,
					args: {
						id2,
						compilerName,
						hookName,
						pluginName
					}
				});
				const callback = args.pop();
				fn(
					...args,
					/**
					 * @param {...EXPECTED_ANY[]} r result
					 */
					(...r: any[]) => {
						tracer.endAsync({
							name,
							id,
							args: {
								id2,
								compilerName,
								hookName,
								pluginName
							}
						});
						callback(...r);
					}
				);
			};
		case "sync":
			return (...args: any[]) => {
				const id2 = tracer.counter++;
				// Do not instrument ourself due to the CPU
				// profile needing to be the last event in the trace.
				if (name === PLUGIN_NAME) {
					return fn(...args);
				}

				tracer.startAsync({
					name,
					id,
					args: {
						id2,
						compilerName,
						hookName,
						pluginName
					}
				});
				let r: any;
				try {
					r = fn(...args);
				} catch (err) {
					tracer.endAsync({
						name,
						id,
						args: {
							id2: compilerName,
							hookName,
							pluginName
						}
					});
					throw err;
				}
				tracer.endAsync({
					name,
					id,
					args: {
						id2,
						compilerName,
						hookName,
						pluginName
					}
				});
				return r;
			};
		default:
			return fn;
	}
};

let compilerId = 0;
export class TraceHookPlugin {
	name = PLUGIN_NAME;
	apply(compiler: Compiler) {
		// FIXME: we need a compiler.id for track child compiler
		const compilerName = compiler.name || (compilerId++).toString();
		// Compiler Hooks
		for (const hookName of Object.keys(compiler.hooks)) {
			const hook = compiler.hooks[hookName as keyof Compiler["hooks"]];
			if (hook) {
				hook.intercept(
					makeInterceptorFor(compilerName, JavaScriptTracer)(hookName)
				);
			}
		}
		compiler.hooks.compilation.tap(
			PLUGIN_NAME,
			(compilation, { normalModuleFactory, contextModuleFactory }) => {
				interceptAllHooksFor(compilation, JavaScriptTracer, "Compilation");
				interceptAllHooksFor(
					normalModuleFactory,
					JavaScriptTracer,
					"Normal Module Factory"
				);
				interceptAllHooksFor(
					contextModuleFactory,
					JavaScriptTracer,
					"Context Module Factory"
				);
			}
		);
	}
}
