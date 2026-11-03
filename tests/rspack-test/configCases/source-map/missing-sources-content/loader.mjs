import path from "node:path";

export default function (code) {
	const kind = this.resourceQuery.slice(1);
	const map = {
		version: 3,
		sources: [path.join(this.context, `original-${kind}.js`)],
		names: [],
		mappings: "AAAA"
	};
	if (kind !== "missing") {
		map.sourcesContent = [kind === "null" ? null : kind === "empty" ? "" : code];
	}
	this.callback(null, code, map);
}
