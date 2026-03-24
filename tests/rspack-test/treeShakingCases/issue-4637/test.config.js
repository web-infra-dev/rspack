const HELPER_ID = '"<PNPM_INNER>/@swc/helpers/esm/_create_class.js"(';
const INDEX_ID = '"./index.js"(';
const UTIL_ID = '"./util.js"(';
const RUNTIME_MARKER = '\n\n},function(__webpack_require__) {';

module.exports = {
	snapshotContent(content) {
		const end = content.indexOf(RUNTIME_MARKER);
		if (end < 0) {
			return content;
		}

		const ids = [INDEX_ID, UTIL_ID, HELPER_ID];
		const starts = ids.map(id => [id, content.indexOf(id)]).filter(([, index]) => index >= 0);
		if (starts.length !== ids.length) {
			return content;
		}

		starts.sort((a, b) => a[1] - b[1]);
		const prefix = content.slice(0, starts[0][1]);
		const suffix = content.slice(end);
		const blocks = new Map(
			starts.map(([id, start], index) => {
				const nextStart = starts[index + 1]?.[1] ?? end;
				return [id, content.slice(start, nextStart)];
			})
		);

		return prefix + ids.map(id => blocks.get(id)).join('') + suffix;
	},
};
