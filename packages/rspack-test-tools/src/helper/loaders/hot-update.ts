import type { TUpdateOptions } from "../../type";

export default function (this: any, c: string) {
	let content = c;
	if (content.includes("NEXT_HMR")) {
		content = `
			${content}
			let __hmr_children__ = [...module.children];
      let __hmr_used_exports__ = __hmr_children__.reduce((res, child) => {
        if (__webpack_module_cache__[child]) {
          res[child] = __webpack_module_cache__[child].exports;
        }
				return res;
			}, {});
			module.hot.accept(__hmr_children__, () => {
				__hmr_children__.forEach((child) => {
					const reexports = __webpack_require__(child);
          for (let key in reexports) {
            if (!__hmr_used_exports__[child]) { continue; }
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

	const options: TUpdateOptions = this.getOptions();
	const items = content.split(/---+\r?\n/g);
	if (items.length <= 1) {
		return content;
	}

	options.totalUpdates = Math.max(options.totalUpdates, items.length);
	options.changedFiles.push(this.resourcePath);
	return items[options.updateIndex];
}
