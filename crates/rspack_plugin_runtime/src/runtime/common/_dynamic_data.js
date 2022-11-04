(function () {
	runtime.installedCssChunks = {};
})();

(function () {
	runtime.chunkHashData = {
		js: __JS__,
		css: __CSS__
	};
})();

(function () {
	runtime.setChunkHashData = function (chunkId, hash, type) {
		return (this.chunkHashData[type][chunkId] = hash);
	};
})();

(function () {
	runtime.__rspack_has_dynamic_chunk__ = function (chunkId, type) {
		return Boolean(
			this.chunkHashData &&
				this.chunkHashData[type] &&
				typeof this.chunkHashData[type][chunkId] !== "undefined"
		);
	};
})();
