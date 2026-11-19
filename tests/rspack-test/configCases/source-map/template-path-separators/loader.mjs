import { fixtures } from "./fixtures.mjs";

export default function () {
	const sources = [...fixtures.map(({ source }) => source), this._module.identifier()];
	this.callback(null, sources.map((_, i) => `exports.value${i} = ${i};`).join("\n"), {
		version: 3,
		// Keep the original path spelling through the loader-map context conversion.
		sources: sources.map(source => `webpack://${source}`),
		sourcesContent: [...fixtures.map((_, i) => `fixture:${i}`), "module-identifier"],
		names: [],
		mappings: sources.map((_, i) => i === 0 ? "AAAA" : "ACAA").join(";")
	});
}
