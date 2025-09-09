module.exports = {
	plugins: [
		{
			apply(
				/**@type {import('@rspack/core').Compiler} */
				compiler
			) {
				compiler.hooks.compilation.tap("test", compilation => {
					compilation.hooks.seal.tap("test", () => {
						compilation.errors.push([
							{
								name: "test error",
								message: "",
								loc: {
									start: {
										line: 0,
										column: 0
									},
									end: {
										line: 0,
										column: 0
									}
								}
							}
						]);
					});
				});
			}
		}
	]
};
