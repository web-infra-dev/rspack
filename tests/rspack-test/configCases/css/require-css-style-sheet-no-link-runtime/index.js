import sheet from "./style.css";

// Keep an async chunk so the ensure-chunk handler is part of the runtime.
export const loadLazy = () => import("./lazy.js");

it("should keep the stylesheet link runtime out of require chunk loading", () => {
	expect(sheet).toBeDefined();
	// The style-sheet runtime is still emitted.
	expect(typeof __webpack_require__.css).toBe("function");
	// But the link/loading runtimes are not, as with webpack.
	expect(__webpack_require__.hmrC && __webpack_require__.hmrC.css).toBeUndefined();
	expect(__webpack_require__.f && __webpack_require__.f.css).toBeUndefined();
});
