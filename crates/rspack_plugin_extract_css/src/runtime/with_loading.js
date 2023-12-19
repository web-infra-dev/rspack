// object to store loaded CSS chunks
var installedCssChunks = {
	__INSTALLED_CHUNKS__
};

__ENSURE_CHUNK_HANDLERS__.miniCss = function(chunkId, promises) {
	var cssChunks = __CSS_CHUNKS__;
	if(installedCssChunks[chunkId]) promises.push(installedCssChunks[chunkId])
	else if(installedCssChunks[chunkId] !== 0 && cssChunks[chunkId])
		promises.push(
			installedCssChunks[chunkId] = loadStylesheet(chunkId).then(
				function() {
					installedCssChunks[chunkId] = 0;
				},
				function(e) {
					delete installedCssChunks[chunkId];
					throw e;
				}
			)
		)
}
