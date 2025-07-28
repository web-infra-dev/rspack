import type { Compiler } from "../index";

type RequireFn = (id: string) => Promise<any>;

export function nonWebpackRequire(compiler: Compiler): RequireFn {
	return async function (id: string) {
		if (IS_BROWSER) {
			// Why is IS_BROWSER used here:
			// Loading modules in @rspack/browser is difference from the @rspack/core.
			// 1. It resolves the JavaScript in the memfs with Node.js resolution algorithm rather than in the host filesystem.
			// 2. It customizes how to evaluate CJS/ESM because there's no `require` any more.
			const resolver = compiler._lastCompilation!.resolverFactory.get("normal");
			return new Promise((resolve, reject) => {
				resolver.resolve({}, "", id, {}, (err, loaderPath) => {
					if (err) {
						reject(err);
						return;
					}
					if (!loaderPath) {
						reject(`Cannot find loader of ${id}`);
						return;
					}
					const inputFileSystem = compiler.inputFileSystem!;
					inputFileSystem.readFile(loaderPath, {}, (err, data) => {
						if (err) {
							reject(err);
							return;
						}

						const loaderCode = data?.toString() || "";

						// 1. Assume it's a cjs
						try {
							// Use `new Function` to emulate CJS
							const module = { exports: {} };
							const exports = module.exports;
							const createRequire = () => {
								throw new Error(
									"@rspack/browser doesn't support `require` in loaders yet"
								);
							};

							// rslint-disable no-implied-eval
							const wrapper = new Function(
								"module",
								"exports",
								"require",
								loaderCode
							);

							wrapper(module, exports, createRequire);
							resolve(module.exports);
						} catch {
							// 2. Assume it's an esm
							// Use `import(base64code)` to load ESM
							const dataUrl = `data:text/javascript;base64,${btoa(loaderCode)}`;
							try {
								// biome-ignore lint/security/noGlobalEval: use `eval("import")` rather than `import` to suppress the warning in @rspack/browser
								const modulePromise = eval(`import("${dataUrl}")`);
								modulePromise.then(resolve);
							} catch (e) {
								reject(e);
							}
						}
					});
				});
			});
		}

		// In `@rspack/core`, just use Node.js require
		return require(id);
	};
}
