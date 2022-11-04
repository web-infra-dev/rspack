function __rspack_dynamic_require__(chunkIds) {
	return Promise.all(
		chunkIds
			.map(
				function (chunkId) {
					return Object.keys(this)
						.filter(function (key) {
							return key.indexOf("rspack_load_dynamic") > 0;
						})
						.reduce(
							function (promises, key) {
								this[key](chunkId, promises);
								return promises;
							}.bind(this),
							[]
						);
				}.bind(this)
			)
			.reduce(function (prev, cur) {
				return prev.concat(cur);
			}, [])
	);
}

// mount register dynamic require
(function () {
	runtime.__rspack_dynamic_require__ = __rspack_dynamic_require__;
})();
