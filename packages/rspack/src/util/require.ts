import fs from "node:fs";
import { async as resolveAsync } from "@rspack/binding";

type RequireFn = (id: string) => Promise<any>;

export function nonWebpackRequire(): RequireFn {
	return async function (id: string) {
		if (IS_BROWSER) {
			// Why is IS_BROWSER used here:
			// Loading modules in @rspack/browser is difference from the @rspack/core.
			// 1. It resolves the JavaScript in the memfs with Node.js resolution algorithm rather than in the host filesystem.
			// 2. It customizes how to evaluate CJS/ESM because there's no `require` any more.
			return new Promise((resolve, reject) => {
				resolveAsync("", id)
					.then(({ path: loaderPath }) => {
						if (!loaderPath) {
							reject(`Cannot find loader of ${id}`);
							return;
						}
						fs.readFile(loaderPath, {}, (err, data) => {
							if (err) {
								reject(err);
								return;
							}

							// Only custom esm loader is supported.
							const loaderCode = data?.toString() || "";
							const codeUrl = URL.createObjectURL(
								new Blob([loaderCode], { type: "text/javascript" })
							);
							try {
								// We have to use `eval` to prevent this dynamic import being handled by any bundler.
								// Applications should config their CSP to allow `unsafe-eval`.
								// In the future, we may find a better way to handle this, such as user-injected module executor.
								// biome-ignore lint/security/noGlobalEval: use `eval("import")` rather than `import` to suppress the warning in @rspack/browser
								const modulePromise = eval(
									`import("${codeUrl}")`
								) as Promise<unknown>;
								modulePromise.then(module => {
									URL.revokeObjectURL(codeUrl);
									resolve(module);
								});
							} catch (e) {
								reject(e);
							}
						});
					})
					.catch(err => reject(err));
			});
		}

		// In `@rspack/core`, just use Node.js require
		return require(id);
	};
}
