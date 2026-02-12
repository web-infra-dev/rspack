const NormalModule = require("@rspack/core").NormalModule;
const path = require("path")

module.exports = {
	module: {
		rules: [
			{
				test: /\.png$/,
				type: "asset",
				parser: {
					dataUrlCondition: (source, { filename, module }) => {
						expect(source).toBeInstanceOf(Buffer);
						expect(filename).toBe(
							path.resolve(__dirname, "test.png")
						);
						expect(module).toBeInstanceOf(NormalModule);
						return true;
					}
				}
			}
		]
	}
};
