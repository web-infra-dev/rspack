import value from "./file";

it("should accept a dependencies and require a new value", async () => {
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(2);
	await NEXT_HMR();
	expect(value).toBe(1);
	await NEXT_HMR();
	expect(value).toBe(3);
});

let children = [...module.children];
let usedExports = children.reduce((res, child) => {
	res[child] = __webpack_module_cache__[child].exports;
	return res;
}, {});
module.hot.accept(children, () => {
	children.forEach((child) => {
		const reexports = __webpack_require__(child);
		for (let key in reexports) {
			Object.defineProperty(usedExports[child], key, {
				configurable: true,
				enumerable: true,
				get: () => reexports[key]
			});
		}
	});
});
