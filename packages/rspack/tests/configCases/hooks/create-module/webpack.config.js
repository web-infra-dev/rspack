const fs = require("fs");
const path = require("path");

/** @type {import('@rspack/core').Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.normalModuleFactory.tap("mock-plugin", nmf => {
					nmf.hooks.createModule.tap("mock-plugin", createData => {
						fs.writeFileSync(
							path.resolve(__dirname, "./dist/createData.json"),
							JSON.stringify(createData, null, 2)
						);
					});
				});
			}
		}
	]
};
