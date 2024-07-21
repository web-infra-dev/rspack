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
import type { LoaderObject } from ".";

type ModuleObject = {
	default?: Function;
	pitch?: Function;
	raw?: boolean;
};
type LoaderModule = ModuleObject | Function;

import LoaderLoadingError from "./LoaderLoadingError";
var url: undefined | typeof Url = undefined;

export default function loadLoader(
	loader: LoaderObject,
	callback: (err: unknown) => void
): void {
	if (loader.type === "module") {
		try {
			if (url === undefined) url = require("node:url");
			var loaderUrl = url!.pathToFileURL(loader.path);
			var modulePromise = eval(
				"import(" + JSON.stringify(loaderUrl.toString()) + ")"
			);
			modulePromise.then((module: LoaderModule) => {
				handleResult(loader, module, callback);
			}, callback);
			return;
		} catch (e) {
			callback(e);
		}
	} else {
		try {
			var module = require(loader.path);
		} catch (e) {
			// it is possible for node to choke on a require if the FD descriptor
			// limit has been reached. give it a chance to recover.
			// @ts-expect-error
			if (e instanceof Error && e.code === "EMFILE") {
				var retry = loadLoader.bind(null, loader, callback);
				if (typeof setImmediate === "function") {
					// node >= 0.9.0
					return void setImmediate(retry);
				} else {
					// node < 0.9.0
					return process.nextTick(retry);
				}
			}
			return callback(e);
		}
		return handleResult(loader, module, callback);
	}
}

function handleResult(
	loader: LoaderObject,
	module: LoaderModule,
	callback: (err?: unknown) => void
): void {
	if (typeof module !== "function" && typeof module !== "object") {
		return callback(
			new LoaderLoadingError(
				"Module '" +
					loader.path +
					"' is not a loader (export function or es6 module)"
			)
		);
	}
	loader.normal = typeof module === "function" ? module : module.default;
	loader.pitch = (module as ModuleObject).pitch;
	loader.raw = (module as ModuleObject).raw;
	if (
		typeof loader.normal !== "function" &&
		typeof loader.pitch !== "function"
	) {
		return callback(
			new LoaderLoadingError(
				"Module '" +
					loader.path +
					"' is not a loader (must have normal or pitch function)"
			)
		);
	}
	callback();
}
