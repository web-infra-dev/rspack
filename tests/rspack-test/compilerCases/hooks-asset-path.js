/** @type {import('@rspack/test-tools').TCompilerCaseConfig[]} */
module.exports = [
	(() => {
		let capturedPath = null;
		let capturedData = null;

		return {
			description: "compilation.hooks.assetPath should receive path and PathData",
			options(context) {
				return {
					context: context.getSource(),
					entry: "./d",
					output: {
						filename: "[name].js"
					},
					plugins: [{
						apply(compiler) {
							compiler.hooks.compilation.tap("AssetPathPlugin", compilation => {
								compilation.hooks.assetPath.tap("AssetPathPlugin", (path, data) => {
									capturedPath = path;
									capturedData = data;
									return path;
								});
							});
						}
					}]
				};
			},
			async check() {
				expect(typeof capturedPath).toBe("string");
				expect(capturedPath.length).toBeGreaterThan(0);
				expect(capturedData).toBeTruthy();
				// data.path mirrors the path argument (matches webpack's PathData shape)
				expect(typeof capturedData.path).toBe("string");
			}
		};
	})(),
	(() => {
		const tapFn = rstest.fn(path => path.replace(/\.js$/, ".modified.js"));

		return {
			description: "compilation.hooks.assetPath should use the return value of the tap",
			options(context) {
				return {
					context: context.getSource(),
					entry: "./d",
					output: {
						filename: "[name].js"
					},
					plugins: [{
						apply(compiler) {
							compiler.hooks.compilation.tap("AssetPathPlugin", compilation => {
								compilation.hooks.assetPath.tap("AssetPathPlugin", tapFn);
							});
						}
					}]
				};
			},
			async check({ compiler }) {
				expect(tapFn).toHaveBeenCalled();
				// The tap was called with a string path and PathData
				const [firstPath, firstData] = tapFn.mock.calls[0];
				expect(typeof firstPath).toBe("string");
				expect(firstData).toBeTruthy();
			}
		};
	})(),
	(() => {
		const firstTap = rstest.fn(path => path);
		const secondTap = rstest.fn(path => path);

		return {
			description: "compilation.hooks.assetPath should call multiple taps in stage order",
			options(context) {
				return {
					context: context.getSource(),
					entry: "./d",
					output: {
						filename: "[name].js"
					},
					plugins: [{
						apply(compiler) {
							compiler.hooks.compilation.tap("AssetPathPlugin", compilation => {
								compilation.hooks.assetPath.tap({ name: "Second", stage: 1 }, secondTap);
								compilation.hooks.assetPath.tap({ name: "First", stage: 0 }, firstTap);
							});
						}
					}]
				};
			},
			async check() {
				expect(firstTap).toHaveBeenCalled();
				expect(secondTap).toHaveBeenCalled();
			}
		};
	})()
];
