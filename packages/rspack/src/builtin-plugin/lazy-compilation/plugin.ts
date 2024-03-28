import type { Compiler } from "../..";

import { BuiltinLazyCompilationPlugin } from "./lazyCompilation";
import getBackend, {
	moduleImpl,
	dispose,
	LazyCompilationDefaultBackendOptions
} from "./backend";
import { RawRegexMatcher } from "@rspack/binding";

export default class LazyCompilationPlugin {
	cacheable: boolean;
	entries: boolean;
	imports: boolean;
	test?: RawRegexMatcher;
	backend?: LazyCompilationDefaultBackendOptions;

	constructor(
		cacheable: boolean,
		entries: boolean,
		imports: boolean,
		test?: RawRegexMatcher,
		backend?: LazyCompilationDefaultBackendOptions
	) {
		this.cacheable = cacheable;
		this.entries = entries;
		this.imports = imports;
		this.test = test;
		this.backend = backend;
	}

	apply(compiler: Compiler) {
		const backend = getBackend({
			...this.backend,
			client: require.resolve(
				`../../../hot/lazy-compilation-${
					compiler.options.externalsPresets.node ? "node" : "web"
				}.js`
			)
		});

		new BuiltinLazyCompilationPlugin(
			moduleImpl,
			this.cacheable,
			this.entries,
			this.imports,
			this.test
		).apply(compiler);

		let initialized = false;
		compiler.hooks.beforeCompile.tapAsync(
			"LazyCompilationPlugin",
			(_params, callback) => {
				if (initialized) return callback();
				backend(compiler, (err, result) => {
					if (err) return callback(err);
					initialized = true;
					callback();
				});
			}
		);
		compiler.hooks.shutdown.tapAsync("LazyCompilationPlugin", callback => {
			dispose(callback);
		});
	}
}

export { LazyCompilationPlugin };
