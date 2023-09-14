/**
 * The following code is modified based on
 * https://github.com/webpack/webpack/blob/4b4ca3b/lib/node/NodeTemplatePlugin.js
 *
 * MIT Licensed
 * Author Tobias Koppers @sokra
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/webpack/webpack/blob/main/LICENSE
 */

import { Compiler } from "../Compiler";
import {
	CommonJsChunkFormatPlugin,
	EnableChunkLoadingPlugin
} from "../builtin-plugin";

export type NodeTemplatePluginOptions = { asyncChunkLoading?: boolean };

export default class NodeTemplatePlugin {
	constructor(private _options: NodeTemplatePluginOptions = {}) {}

	apply(compiler: Compiler) {
		const chunkLoading = this._options.asyncChunkLoading
			? "async-node"
			: "require";
		compiler.options.output.chunkLoading = chunkLoading;
		new CommonJsChunkFormatPlugin().apply(compiler);
		new EnableChunkLoadingPlugin(chunkLoading).apply(compiler);
	}
}
