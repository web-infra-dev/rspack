export default function (c: string) {
	let content = c;
	if (
		content.includes("NEXT_HMR") &&
		!content.includes("module.hot.accept") &&
		!content.includes("import.meta.webpackHot.accept")
	) {
		content = `
			${content}
			// let __hmr_children__ = [...module.children];
      // let __hmr_used_exports__ = __hmr_children__.reduce((res, child) => {
      //   if (__webpack_require__.c[child]) {
      //     res[child] = __webpack_require__.c[child].exports;
      //   }
			// 	return res;
			// }, {});
			// module.hot.accept(__hmr_children__, () => {
			// 	__hmr_children__.forEach((child) => {
			// 		const reexports = __webpack_require__(child);
			// 		if (typeof reexports === "object") {
			// 			for (let key in reexports) {
			// 				if (!__hmr_used_exports__[child]) { continue; }
			// 				Object.defineProperty(__hmr_used_exports__[child], key, {
			// 					configurable: true,
			// 					enumerable: true,
			// 					get: () => reexports[key]
			// 				});
			// 			}
			// 		}
			// 	});
			// });
		`;
	}
	return content.replace(/NEXT_HMR/g, "NEXT_HMR.bind(null, module)");
}
