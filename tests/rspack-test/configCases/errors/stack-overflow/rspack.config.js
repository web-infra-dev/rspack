module.exports = {
	plugins: [
		{
			apply(
				/**@type {import('@rspack/core').Compiler} */
				compiler
			) {
				compiler.hooks.compilation.tap("stack-overflow-test", compilation => {
					compilation.hooks.processAssets.tap("stack-overflow-test", () => {
						// Create a recursive function that will cause a stack overflow
						function recursiveFunction() {
							recursiveFunction();
						}

						// Call the recursive function to trigger the stack overflow
						recursiveFunction();
					});
				});
			}
		}
	]
};
