function install_chunk(chunk) {
	var moreModules = chunk.modules,
		chunkIds = chunk.ids;
	for (var moduleId in moreModules) {
		if (this.checkById(moreModules, moduleId)) {
			this.installedModules[moduleId] = moreModules[moduleId];
		}
	}
	for (var i = 0; i < chunkIds.length; i++)
		this.installedChunks[chunkIds[i]] = 1;
}

// mount load dynamic js
(function () {
	runtime.install_chunk = install_chunk;
})();
