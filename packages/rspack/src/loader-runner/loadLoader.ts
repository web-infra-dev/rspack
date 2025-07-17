/**
 * The following code is from
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */

import type Url from "node:url";
import type { LoaderDefinitionFunction } from "../config";
import type {
	LoaderContext,
	PitchLoaderDefinitionFunction
} from "../config/adapterRuleUse";
import type { LoaderObject } from ".";
import LoaderLoadingError from "./LoaderLoadingError";
import { InputFileSystem } from "../util/fs";

type ModuleObject = {
	default?: LoaderDefinitionFunction;
	pitch?: PitchLoaderDefinitionFunction;
	raw?: boolean;
};
type LoaderModule = ModuleObject | Function;

let url: undefined | typeof Url;

export default function loadLoader(
	loader: LoaderObject,
	loaderContext: LoaderContext,
	callback: (err: unknown) => void
): void {
	if (IS_BROWSER) {
		// Why is IS_BROWSER used here:
		// Loading loaders in @rspack/browser is difference from the @rspack/core.
		// 1. It resolves the JavaScript in the memfs with Node.js resolution algorithm rather than in the host filesystem.
		// 2. It customizes how to evaluate CJS/ESM because there's no `require` any more.
		return loadLoaderInBrowser(loader, loaderContext, callback);
	}

	if (loader.type === "module") {
		try {
			if (url === undefined) url = require("node:url");
			const loaderUrl = url!.pathToFileURL(loader.path);
			const modulePromise = import(loaderUrl.toString());
			modulePromise.then((module: LoaderModule) => {
				handleResult(loader, module, callback);
			}, callback);
			return;
		} catch (e) {
			callback(e);
		}
	} else {
		let module: any;
		try {
			module = require(loader.path);
		} catch (e) {
			// it is possible for node to choke on a require if the FD descriptor
			// limit has been reached. give it a chance to recover.
			if (
				e instanceof Error &&
				(e as NodeJS.ErrnoException).code === "EMFILE"
			) {
				const retry = loadLoader.bind(null, loader, loaderContext, callback);
				return void setImmediate(retry);
			}
			return callback(e);
		}
		return handleResult(loader, module, callback);
	}
}

const loadLoaderInBrowser: typeof loadLoader = (
	loader,
	loaderContext,
	callback
) => {
	loaderContext.resolve(
		loaderContext._compiler.context,
		loader.path,
		(err, loaderPath) => {
			if (err) {
				callback(err);
				return;
			}
			if (!loaderPath) {
				callback(`Cannot find loader of ${loader.path}`);
				return;
			}
			const inputFileSystem = loaderContext.fs as InputFileSystem;
			inputFileSystem.readFile(loaderPath, {}, (err, data) => {
				if (err) {
					callback(err);
					return;
				}
				// Currently only esm loader is supported
				const loaderCode = data?.toString() || "";
				const dataUrl = `data:text/javascript;base64,${btoa(loaderCode)}`;
				try {
					// biome-ignore lint/security/noGlobalEval: use `eval("import")` rather than `import` to suppress the warning in @rspack/browser
					const modulePromise = eval(`import("${dataUrl}")`);
					modulePromise.then((module: LoaderModule) => {
						handleResult(loader, module, callback);
					}, callback);
					return;
				} catch (e) {
					callback(e);
				}
			});
		}
	);
};

function handleResult(
	loader: LoaderObject,
	module: LoaderModule,
	callback: (err?: unknown) => void
): void {
	if (typeof module !== "function" && typeof module !== "object") {
		return callback(
			new LoaderLoadingError(
				`Module '${loader.path}' is not a loader (export function or es6 module)`
			)
		);
	}
	loader.normal = typeof module === "function" ? module : module.default;
	loader.pitch = (module as ModuleObject).pitch;
	loader.raw = (module as ModuleObject).raw;
	if (!loader.pitch) {
		loader.noPitch = true;
	}
	if (
		typeof loader.normal !== "function" &&
		typeof loader.pitch !== "function"
	) {
		return callback(
			new LoaderLoadingError(
				`Module '${loader.path}' is not a loader (must have normal or pitch function)`
			)
		);
	}
	callback();
}
