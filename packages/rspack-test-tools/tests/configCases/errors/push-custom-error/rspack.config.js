module.exports = {
	plugins: [
		{
			apply(
				/**@type {import('@rspack/core').Compiler} */
				compiler
			) {
				compiler.hooks.compilation.tap("test", compilation => {
					compilation.hooks.seal.tap("test", () => {
						const error = new Error("");
						error.name = "test error";
						error.loc = {
							start: {
								line: 0,
								column: 0
							},
							end: {
								line: 0,
								column: 0
							}
						};
						compilation.errors.push([error]);
					});
				});
			}
		}
	]
};
