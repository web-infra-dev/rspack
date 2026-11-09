var __webpack_modules__ = { prebuilt: () => "prebuilt" };
var __rspack_modules = { prebuilt: () => "prebuilt" };

it("keeps entry-local module tables separate from the runtime", () => {
	expect(__webpack_modules__.prebuilt()).toBe("prebuilt");
	expect(__rspack_modules.prebuilt()).toBe("prebuilt");
	expect(require("./dependency")).toBe("dependency");
});
