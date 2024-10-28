// @ts-nocheck
exports.checkChunkModules = function checkChunkModules(
	statsJson,
	chunkModulesMap,
	strict = true
) {
	for (const chunkId of Reflect.ownKeys(chunkModulesMap)) {
		const chunk = getChunk(statsJson, chunkId);

		const expectedModules = chunkModulesMap[chunkId];
		const chunkModules = chunk.modules.map(m => m.identifier);
		if (strict) {
			expectedModules.length === chunkModules.length;
		}

		for (const module of expectedModules) {
			if (!chunkModules.find(moduleId => moduleId.includes(module))) {
				throw new Error(
					`chunk ${chunkId} has no module contains id: ${module}`
				);
			}
		}
	}

	return true;
};

exports.checkChunkRuntime = function checkChunkModules(
	statsJson,
	chunkModulesMap,
	strict = true
) {
	for (const chunkId of Reflect.ownKeys(chunkModulesMap)) {
		const chunk = getChunk(statsJson, chunkId);

		const expectedRuntime = chunkModulesMap[chunkId];
		const chunkRuntime = chunk.runtime;

		if (strict) {
			expectedRuntime.length === chunkRuntime.length;
		}

		for (let i = 0; i < expectedRuntime.length; i++) {
			const expected = expectedRuntime[i];
			const rt = chunkRuntime[i];
			if (expected !== rt) {
				throw new Error(
					`chunk ${chunkId} runtime not equal, expected: ${expectedRuntime}, but got: ${chunkRuntime}`
				);
			}
		}
	}

	return true;
};

function getChunk(statsJson, id) {
	const chunk = statsJson.chunks.find(chunk => chunk.id.includes(id));

	if (!chunk) {
		throw new Error(`cannot find chunk with id: ${id}`);
	}

	return chunk;
}
