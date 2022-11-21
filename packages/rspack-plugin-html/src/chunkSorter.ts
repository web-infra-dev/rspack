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
