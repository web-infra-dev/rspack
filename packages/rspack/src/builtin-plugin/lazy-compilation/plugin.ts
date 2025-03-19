import type { Module } from "../../Module";

import type { Compiler } from "../..";
import getBackend, {
	type LazyCompilationDefaultBackendOptions
} from "./backend";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";

export default class LazyCompilationPlugin {
	cacheable: boolean;
	entries: boolean;
	imports: boolean;
	test?: RegExp | ((module: Module) => boolean);
	backend?: LazyCompilationDefaultBackendOptions;

	constructor(
		cacheable: boolean,
		entries: boolean,
		imports: boolean,
		test?: RegExp | ((module: Module) => boolean),
		backend?: LazyCompilationDefaultBackendOptions
	) {
		this.cacheable = cacheable;
		this.entries = entries;
		this.imports = imports;
		this.test = test;
		this.backend = backend;
	}

	apply(compiler: Compiler) {
		const { state, backend } = getBackend({
			client: require.resolve(
				`../hot/lazy-compilation-${
					compiler.options.externalsPresets.node ? "node" : "web"
				}.js`
			),
			...this.backend
		});

		new BuiltinLazyCompilationPlugin(
			args => state.module(args),
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
			state.dispose(callback);
		});
	}
}

export { LazyCompilationPlugin };
