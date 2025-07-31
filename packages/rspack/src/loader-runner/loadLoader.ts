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
import type { PitchLoaderDefinitionFunction } from "../config/adapterRuleUse";
import type { LoaderObject } from ".";
import LoaderLoadingError from "./LoaderLoadingError";

type ModuleObject = {
	default?: LoaderDefinitionFunction;
	pitch?: PitchLoaderDefinitionFunction;
	raw?: boolean;
};
type LoaderModule = ModuleObject | Function;

let url: undefined | typeof Url;

export default function loadLoader(
	loader: LoaderObject,
	callback: (err: unknown) => void
): void {
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
				const retry = loadLoader.bind(null, loader, callback);
				return void setImmediate(retry);
			}
			callback(e);
			return;
		}
		handleResult(loader, module, callback);
		return;
	}
}

function handleResult(
	loader: LoaderObject,
	module: LoaderModule,
	callback: (err?: unknown) => void
): void {
	if (typeof module !== "function" && typeof module !== "object") {
		callback(
			new LoaderLoadingError(
				`Module '${loader.path}' is not a loader (export function or es6 module)`
			)
		);
		return;
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
		callback(
			new LoaderLoadingError(
				`Module '${loader.path}' is not a loader (must have normal or pitch function)`
			)
		);
		return;
	}
	callback();
}
