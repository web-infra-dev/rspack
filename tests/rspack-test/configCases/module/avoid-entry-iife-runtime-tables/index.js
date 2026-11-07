var __webpack_modules__ = { prebuilt: () => "prebuilt" };
var __webpack_module_cache__ = { cached: true };
var __rspack_modules = { prebuilt: () => "prebuilt" };
var __rspack_module_cache = { cached: true };

it("should not let the entry's top-level declarations replace the runtime tables", () => {
	expect(__webpack_modules__.prebuilt()).toBe("prebuilt");
	expect(__webpack_module_cache__.cached).toBe(true);
	expect(__rspack_modules.prebuilt()).toBe("prebuilt");
	expect(__rspack_module_cache.cached).toBe(true);
	expect(require("./dependency")).toBe("dependency");
});
