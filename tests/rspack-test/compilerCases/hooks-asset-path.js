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
				// data is JsPathData — path context separate from the waterfall path arg
				expect(capturedData).toBeTruthy();
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
		const callOrder = [];
		const firstTap = rstest.fn(path => { callOrder.push("first"); return path; });
		const secondTap = rstest.fn(path => { callOrder.push("second"); return path; });

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
								// Registered in reverse order — stage should determine execution order
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
				// stage 0 (First) must run before stage 1 (Second)
				expect(callOrder.indexOf("first")).toBeLessThan(callOrder.indexOf("second"));
			}
		};
	})()
];
