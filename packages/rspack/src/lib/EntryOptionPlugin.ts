/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/EntryOptionPlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import assert from "assert";

import type { Compiler, EntryDescriptionNormalized, EntryNormalized } from "..";
import {
	DynamicEntryPlugin,
	type EntryOptions,
	EntryPlugin
} from "../builtin-plugin";

export default class EntryOptionPlugin {
	apply(compiler: Compiler) {
		compiler.hooks.entryOption.tap("EntryOptionPlugin", (context, entry) => {
			EntryOptionPlugin.applyEntryOption(compiler, context, entry);
			return true;
		});
	}

	static applyEntryOption(
		compiler: Compiler,
		context: string,
		entry: EntryNormalized
	) {
		if (typeof entry === "function") {
			new DynamicEntryPlugin(context, entry).apply(compiler);
		} else {
			for (const name of Object.keys(entry)) {
				const desc = entry[name];
				const options = EntryOptionPlugin.entryDescriptionToOptions(
					compiler,
					name,
					desc
				);
				assert(
					desc.import !== undefined,
					"desc.import should not be `undefined` once `EntryOptionPlugin.applyEntryOption` is called"
				);
				for (const entry of desc.import) {
					new EntryPlugin(context, entry, options).apply(compiler);
				}
			}
		}
	}

	static entryDescriptionToOptions(
		compiler: Compiler,
		name: string,
		desc: EntryDescriptionNormalized
	): EntryOptions {
		const options = {
			name,
			filename: desc.filename,
			runtime: desc.runtime,
			// layer: desc.layer,
			dependOn: desc.dependOn,
			baseUri: desc.baseUri,
			publicPath: desc.publicPath,
			chunkLoading: desc.chunkLoading,
			asyncChunks: desc.asyncChunks,
			// wasmLoading: desc.wasmLoading,
			library: desc.library
		};
		// if (desc.layer !== undefined && !compiler.options.experiments.layers) {
		// 	throw new Error(
		// 		"'entryOptions.layer' is only allowed when 'experiments.layers' is enabled"
		// 	);
		// }
		// if (desc.chunkLoading) {
		// 	const EnableChunkLoadingPlugin = require("./javascript/EnableChunkLoadingPlugin");
		// 	EnableChunkLoadingPlugin.checkEnabled(compiler, desc.chunkLoading);
		// }
		// if (desc.wasmLoading) {
		// 	const EnableWasmLoadingPlugin = require("./wasm/EnableWasmLoadingPlugin");
		// 	EnableWasmLoadingPlugin.checkEnabled(compiler, desc.wasmLoading);
		// }
		// if (desc.library) {
		// 	const EnableLibraryPlugin = require("./library/EnableLibraryPlugin");
		// 	EnableLibraryPlugin.checkEnabled(compiler, desc.library.type);
		// }
		return options;
	}
}
