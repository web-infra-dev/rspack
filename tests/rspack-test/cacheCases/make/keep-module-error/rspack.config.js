const rspack = require("@rspack/core");

let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	optimization: {
		minimize: false
	},
	cache: {
		type: "persistent"
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tapPromise("PLUGIN", async stats => {
					const { errors } = stats.toJson({ errors: true });
					if (index === 0) {
						expect(errors.length).toBe(1);
						expect(errors[0].message).toMatch("LoaderError");
					} else {
						// TODO should be same as index 0, change it after
						// write error to module.diagnostic
						expect(errors.length).toBe(0);
					}
					index++;
				});
			}
		}
	]
};
