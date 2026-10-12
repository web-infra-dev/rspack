import assert from "node:assert/strict";

export default function (content, map) {
	const mode = this.resourceQuery.slice(1);
	if (mode === "drop") {
		assert.equal(map, undefined);
	} else {
		assert.equal(map.version, 3);
		switch (mode) {
			case "field":
				assert.equal(map.debugId, "loader-map-debug-id");
				break;
			case "array":
				assert.equal(map.sources[0], "modified.js");
				assert.equal(map.sourcesContent[0], "// modified source content");
				assert(map.names.includes("added-name"));
				break;
			case "descriptor":
				assert.equal(map.sources[0], "descriptor.js");
				assert.equal(map.debugId, "lazy-source-map:descriptor");
				break;
			case "delete":
				assert.equal(map.sourcesContent, undefined);
				break;
			case "replace":
				assert.equal(map.debugId, "replacement-debug-id");
				break;
		}
		this._compiler.__sourceMapTest.protectedMaps.add(map);
	}
	this.callback(null, content, map);
}
