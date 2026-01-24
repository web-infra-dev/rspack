/** @type {import("@rspack/test-tools").THookCaseConfig} */
module.exports = {
	description: "should throw helpful error for unsupported compilation hooks",
	options() {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.compilation.tap("test", compilation => {
							// Test accessing an unsupported hook that exists in webpack but not rspack
							expect(() => {
								compilation.hooks.nonExistentWebpackHook.tap("test", () => {});
							}).toThrow(/Compilation\.hooks\.nonExistentWebpackHook is not supported in rspack/);

							// Verify the error message includes the available hooks list
							try {
								compilation.hooks.nonExistentWebpackHook;
							} catch (e) {
								expect(e.message).toContain("Available compilation hooks:");
								expect(e.message).toContain("processAssets");
								expect(e.message).toContain("seal");
							}
						});
					}
				}
			]
		};
	}
};
