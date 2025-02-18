import type { JsModule } from "@rspack/binding";

import type { Compiler, LazyCompilationOptions } from "../..";
import getBackend, {
	dispose,
	moduleImpl,
	type LazyCompilationDefaultBackendOptions
} from "./backend";
import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";
import { lazyCompilationMiddleware } from "./middleware";

export default class LazyCompilationPlugin {
	cacheable: boolean;
	entries: boolean;
	imports: boolean;
	test?: RegExp | ((m: JsModule) => boolean);
	backend?: LazyCompilationDefaultBackendOptions;
	options: LazyCompilationOptions;

	constructor(
		userOptions: LazyCompilationOptions,
		cacheable: boolean,
		entries: boolean,
		imports: boolean,
		test?: RegExp | ((m: JsModule) => boolean)
	) {
		const options = userOptions ?? {};
		this.options = options;
		this.backend = options?.backend;
		this.cacheable = cacheable;
		this.entries = entries;
		this.imports = imports;
		this.test = test;
	}

	apply(compiler: Compiler) {
		const backend = getBackend(this.options, {
			client: require.resolve(
				`../hot/lazy-compilation-${
					compiler.options.externalsPresets.node ? "node" : "web"
				}.js`
			),
			...this.backend
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
			dispose(callback);
		});
	}
}

export { LazyCompilationPlugin, lazyCompilationMiddleware };
