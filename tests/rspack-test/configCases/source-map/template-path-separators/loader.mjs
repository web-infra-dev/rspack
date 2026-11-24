export function pitch() {
  if (!this.resourcePath.startsWith("custom:")) return;
  this.callback(null, "module.exports = 42;", {
    version: 3,
    sources: [`webpack://${this._module.identifier()}`],
    sourcesContent: ["custom-source"],
    names: [],
    mappings: "AAAA",
  });
}

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
