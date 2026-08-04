import value from "./mod.js";

it("registers the css hmr handler when native css is configured but not yet imported", async () => {
	expect(value).toBe("no-css");
	// Root cause of #14747: the css hmr handler must be baked into the initial
	// runtime chunk even though the initial compilation has no css module yet,
	// otherwise the first css added by a later hot update can never be requested.
	expect(__webpack_require__.hmrC.css).toBeInstanceOf(Function);

	// Adding the first css must not throw when the handler runs (it needs the
	// css chunk filename runtime to be provided alongside the handler).
	await NEXT_HMR();
});

module.hot.accept("./mod.js");
