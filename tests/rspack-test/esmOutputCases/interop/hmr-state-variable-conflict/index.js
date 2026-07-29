const __rspack_hmr_s_module = "application";

it("should avoid conflicts with HMR state variables", () => {
	expect(__rspack_hmr_s_module).toBe("application");
});

export { __rspack_hmr_s_module };
