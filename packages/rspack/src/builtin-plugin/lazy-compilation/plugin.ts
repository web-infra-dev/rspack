import type { JsModule } from "@rspack/binding";
import type { Compiler, LazyCompilationOptions } from "../..";
import { Module } from "../../Module";
import getBackend, { dispose, moduleImpl } from "./backend";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";
import { lazyCompilationMiddleware } from "./middleware";

const REGISTERED_COMPILERS = new WeakSet();

export default class LazyCompilationPlugin {
	static pluginName = "LazyCompilationPlugin";
	cacheable: boolean;
	entries: boolean;
	imports: boolean;
	test?: RegExp | ((m: JsModule) => boolean);
	options: LazyCompilationOptions;

	constructor(userOptions: LazyCompilationOptions) {
		const options = userOptions ?? {};
		this.options = options;
		// @ts-expect-error cacheable is hidden field
		this.cacheable = options.cacheable ?? true;
		this.entries = options.entries ?? true;
		this.imports = options.imports ?? true;
		const test =
			typeof options.test === "function"
				? (jsModule: JsModule) =>
						(options.test as (jsModule: Module) => boolean)!.call(
							options,
							Module.__from_binding(jsModule)
						)
				: options.test;
		this.test = test;
	}

	apply(compiler: Compiler) {
		if (REGISTERED_COMPILERS.has(compiler)) {
			throw "LazyCompilationPlugin can only be applied once";
		}
		REGISTERED_COMPILERS.add(compiler);

		const backend = getBackend({
			...this.options,
			backend: {
				client: require.resolve(
					`../hot/lazy-compilation-${
						compiler.options.externalsPresets.node ? "node" : "web"
					}.js`
				),
				...this.options.backend
			}
		});

		new BuiltinLazyCompilationPlugin(
			moduleImpl,
			this.cacheable,
			this.entries,
			this.imports,
			this.test
		).apply(compiler);

		// initialize the backend
		let initialized = false;
		const initBackendPromise = new Promise<void>((resolve, reject) => {
			backend(compiler, err => {
				if (err) {
					reject(err);
				} else {
					initialized = true;
					resolve();
				}
			});
		});

		// handle the listen error in `beforeCompile` hook,
		// so that the dev middleware can print the error
		compiler.hooks.beforeCompile.tapAsync(
			"LazyCompilationPlugin",
			(_params, callback) => {
				if (initialized) {
					return callback();
				}

				initBackendPromise
					.then(() => {
						callback();
					})
					.catch(err => {
						const logger = compiler.getInfrastructureLogger(
							"LazyCompilationBackend"
						);
						logger.error("Failed to listen to lazy compilation server.");
						callback(err);
					});
			}
		);

		compiler.hooks.shutdown.tapAsync("LazyCompilationPlugin", callback => {
			REGISTERED_COMPILERS.delete(compiler);
			dispose(callback);
		});
	}
}

export { LazyCompilationPlugin, lazyCompilationMiddleware };
