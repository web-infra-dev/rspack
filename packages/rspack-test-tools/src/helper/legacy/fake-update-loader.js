// @ts-nocheck
const contentMap = {};
module.exports = function (content) {
	// CHANGE: new hmr support methods

	if (content.includes("NEXT_HMR")) {
		content = `
			${content}

			let __hmr_children__ = [...module.children];
			let __hmr_used_exports__ = __hmr_children__.reduce((res, child) => {
				res[child] = __webpack_module_cache__[child].exports;
				return res;
			}, {});
			module.hot.accept(__hmr_children__, () => {
				__hmr_children__.forEach((child) => {
					const reexports = __webpack_require__(child);
					for (let key in reexports) {
						Object.defineProperty(__hmr_used_exports__[child], key, {
							configurable: true,
							enumerable: true,
							get: () => reexports[key]
						});
					}
				});
			});


		`;
	}

	content = content.replace(/NEXT_HMR/g, "NEXT_HMR.bind(null, module)");

	// CHANGE:
	var idx = this.getOptions().updateIndex;
	var items = content.split(/---+\r?\n/g);
	var curIdx = items[idx] ? idx : items.length - 1;
	var oldIdx = contentMap[this.resourcePath];
	if (curIdx !== oldIdx && global.__CHANGED_FILES__) {
		global.__CHANGED_FILES__.set(this.resourcePath, items.length);
	}
	contentMap[this.resourcePath] = curIdx;
	if (items.length > 1) {
		this.cacheable(false);
	}
	this.callback(null, items[curIdx]);
};
