/** @type {import("@rspack/test-tools").THookCaseConfig} */
module.exports = {
	description: "should throw helpful error for unsupported compiler hooks",
	options() {
		return {
			plugins: [
				{
					apply(compiler) {
						// Test accessing an unsupported hook that exists in webpack but not rspack
						expect(() => {
							compiler.hooks.nonExistentWebpackHook.tap("test", () => {});
						}).toThrow(/Compiler\.hooks\.nonExistentWebpackHook is not supported in rspack/);

						// Verify the error message includes the available hooks list
						try {
							compiler.hooks.nonExistentWebpackHook;
						} catch (e) {
							expect(e.message).toContain("Available compiler hooks:");
							expect(e.message).toContain("compilation");
							expect(e.message).toContain("emit");
						}
					}
				}
			]
		};
	}
};
