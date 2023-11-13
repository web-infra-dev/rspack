/**
 * The following code is from
 * https://github.com/webpack/loader-runner
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/loader-runner/blob/main/LICENSE
 */

var assert = require("assert");
var LoaderLoadingError = require("./LoaderLoadingError");
var { stringifyLoaderObject } = require(".");
var {
	toBuffer,
	serializeObject,
	isNil,
	toObject,
	stringifyLoaderObject
} = require("../util");
var url;

module.exports = function loadLoader(loader, callback) {
	if (loader.type === "module") {
		try {
			if (url === undefined) url = require("url");
			var loaderUrl = url.pathToFileURL(loader.path);
			var modulePromise = eval(
				"import(" + JSON.stringify(loaderUrl.toString()) + ")"
			);
			modulePromise.then(function (module) {
				handleResult(loader, module, callback);
			}, callback);
			return;
		} catch (e) {
			callback(e);
		}
	} else {
		try {
			var module;

			if (loader.path.startsWith("builtin:")) {
				module = async function (content, sourceMap, additionalData) {
					assert(!this.__internal__context.isPitching);
					const callback = this.async();
					const { runBuiltinLoader } = require("@rspack/binding");
					let options = this.getOptions() ?? {};
					// This is used an hack to tell `builtin:swc-loader` whether to return AST or source.
					this.__internal__context.loaderIndexFromJs = this.loaderIndex;
					try {
						const context = await runBuiltinLoader(
							stringifyLoaderObject(loader),
							JSON.stringify(options),
							Object.assign({}, this.__internal__context, {
								content: isNil(content) ? undefined : toBuffer(content),
								sourceMap: serializeObject(sourceMap),
								additionalData: serializeObject(additionalData)
							})
						);

						this.__internal__context.additionalDataExternal =
							context.additionalDataExternal;
						context.fileDependencies.forEach(this.addDependency);
						context.contextDependencies.forEach(this.addContextDependency);
						context.missingDependencies.forEach(this.addMissingDependency);
						context.buildDependencies.forEach(this.addBuildDependency);
						callback(
							null,
							context.content,
							isNil(context.sourceMap)
								? undefined
								: toObject(context.sourceMap),
							isNil(context.additionalData)
								? undefined
								: toObject(context.additionalData)
						);
						this._compilation.__internal__pushNativeDiagnostics(
							context.diagnosticsExternal
						);
					} catch (e) {
						return callback(e);
					}
				};
				module.pitch = function () {
					// Pitching for builtin loader is not supported
				};
			} else {
				module = require(loader.path);
			}
		} catch (e) {
			// it is possible for node to choke on a require if the FD descriptor
			// limit has been reached. give it a chance to recover.
			if (e instanceof Error && e.code === "EMFILE") {
				var retry = loadLoader.bind(null, loader, callback);
				if (typeof setImmediate === "function") {
					// node >= 0.9.0
					return setImmediate(retry);
				} else {
					// node < 0.9.0
					return process.nextTick(retry);
				}
			}
			return callback(e);
		}
		return handleResult(loader, module, callback);
	}
};

function handleResult(loader, module, callback) {
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
	loader.pitch = module.pitch;
	loader.raw = module.raw;
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
