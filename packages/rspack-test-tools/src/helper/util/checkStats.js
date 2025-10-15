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
		if (strict && expectedModules.length !== chunkModules.length) {
			throw new Error(
				`expect chunk ${chunkId} has ${expectedModules.length} modules: ${expectedModules}\nbut received ${chunkModules.length} modules`
			);
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

function getChunk(statsJson, id) {
	const chunk = statsJson.chunks.find(chunk => chunk.id.includes(id));

	if (!chunk) {
		throw new Error(`cannot find chunk with id: ${id}`);
	}

	return chunk;
}
