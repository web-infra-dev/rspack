const HELPER_ID = '"<PNPM_INNER>/@swc/helpers/esm/_create_class.js"(';
const INDEX_ID = '"./index.js"(';
const UTIL_ID = '"./util.js"(';
const RUNTIME_MARKER = '},function(__webpack_require__) {';

function normalizeModuleBlock(block) {
	return block.trimEnd().replace(/,\s*$/, "");
}

module.exports = {
	snapshotContent(content) {
		const runtimeStart = content.indexOf(RUNTIME_MARKER);
		if (runtimeStart < 0) {
			return content;
		}

		const ids = [INDEX_ID, UTIL_ID, HELPER_ID];
		const starts = ids
			.map(id => [id, content.indexOf(id)])
			.filter(([, index]) => index >= 0);
		if (starts.length !== ids.length) {
			return content;
		}

		starts.sort((a, b) => a[1] - b[1]);
		const prefix = content.slice(0, starts[0][1]);
		const blocks = new Map(
			starts.map(([id, start], index) => {
				const nextStart = starts[index + 1]?.[1] ?? runtimeStart;
				return [id, content.slice(start, nextStart)];
			})
		);

		const modules = ids.map(id => normalizeModuleBlock(blocks.get(id)));
		return `${prefix}${modules.join(",\n")}\n\n${content.slice(runtimeStart)}`;
	},
};
