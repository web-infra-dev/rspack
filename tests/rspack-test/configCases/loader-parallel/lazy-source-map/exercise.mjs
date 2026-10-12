import assert from "node:assert/strict";

export default function (content, map) {
	const mode = this.resourceQuery.slice(1);
	const state = { exercised: 0, protectedMaps: new WeakSet(), retained: [] };
	state.exercised++;
	assert.equal(typeof map, "object");
	if (mode === "unread") {
		state.protectedMaps.add(map);
		state.retained.push(map);
	} else if (mode === "drop") {
		this.callback(null, content);
		return;
	} else {
		assert.equal(map.version, 3);
		assert.equal(map.sources, map.sources);
		switch (mode) {
			case "read":
				assert(Object.keys(map).includes("mappings"));
				assert.equal(Object.getPrototypeOf(map), Object.prototype);
				assert("sources" in map);
				state.protectedMaps.add(map);
				break;
			case "noop":
				map.version = map.version;
				map.sources[0] = map.sources[0];
				Object.defineProperty(map, "version", { value: map.version });
				delete map.absent;
				state.protectedMaps.add(map);
				break;
			case "stringify":
				assert.equal(JSON.parse(JSON.stringify(map)).version, 3);
				state.protectedMaps.add(map);
				break;
			case "freeze":
				Object.freeze(map);
				Object.freeze(map.sources);
				assert.equal(map.sources[0], Object.getOwnPropertyDescriptor(map, "sources").value[0]);
				assert.throws(() => { map.version = 4; }, TypeError);
				break;
			case "field":
				map.debugId = "loader-map-debug-id";
				break;
			case "array":
				map.debugId = "lazy-source-map:array";
				map.sources[0] = "modified.js";
				map.sourcesContent[0] = "// modified source content";
				map.names.push("added-name");
				break;
			case "descriptor":
				Object.getOwnPropertyDescriptor(map, "sources").value[0] = "descriptor.js";
				Object.defineProperty(map, "debugId", { value: "lazy-source-map:descriptor", enumerable: true });
				break;
			case "delete":
				delete map.sourcesContent;
				break;
			case "replace":
				map = { ...map, debugId: "replacement-debug-id" };
				break;
			default:
				throw new Error(`Unknown mode: ${mode}`);
		}
	}
	this.callback(null, content, map);
}
