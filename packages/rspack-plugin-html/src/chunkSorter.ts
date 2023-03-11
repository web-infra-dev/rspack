/**
 * The following code is modified based on
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/lib/chunksorter.js
 *
 * MIT Licensed
 * Author Jan Nicklas
 * Copyright (c) JS Foundation and other contributors
 * https://github.com/jantimon/html-webpack-plugin/blob/d5ce5a8f2d12a2450a65ec51c285dd54e36cd921/LICENSE
 */
import { Compilation } from "@rspack/core";
import { ProcessedOptions } from ".";

const none = (chunks: string[]) => chunks;
const auto = none;
const manual = (
	entryPointNames: string[],
	compilation: Compilation,
	htmlWebpackPluginOptions: ProcessedOptions
) => {
	const chunks = htmlWebpackPluginOptions.chunks;
	if (!Array.isArray(chunks)) {
		return entryPointNames;
	}
	// Remove none existing entries from
	// htmlWebpackPluginOptions.chunks
	return chunks.filter(entryPointName => {
		return compilation.entrypoints.has(entryPointName);
	});
};

const chunkSorter = {
	none,
	auto,
	manual
};

export default chunkSorter;
