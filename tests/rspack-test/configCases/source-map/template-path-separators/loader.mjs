import { fixtures } from "./fixtures.mjs";

export default function () {
	this.callback(null, fixtures.map((_, i) => `exports.value${i} = ${i};`).join("\n"), {
		version: 3,
		// Keep the original path spelling through the loader-map context conversion.
		sources: fixtures.map(({ source }) => `webpack://${source}`),
		sourcesContent: fixtures.map((_, i) => `fixture:${i}`),
		names: [],
		mappings: fixtures.map((_, i) => i === 0 ? "AAAA" : "ACAA").join(";")
	});
}
