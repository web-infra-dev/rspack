var __webpack_module_cache__ = {
	"./dependency.js": { exports: "wrong" }
};
var __rspack_module_cache = {
	"./dependency.js": { exports: "wrong" }
};

it("keeps entry-local caches separate from the runtime", () => {
	expect(require("./dependency")).toBe("dependency");
	expect(Object.keys(__webpack_module_cache__)).toEqual(["./dependency.js"]);
	expect(Object.keys(__rspack_module_cache)).toEqual(["./dependency.js"]);
});
