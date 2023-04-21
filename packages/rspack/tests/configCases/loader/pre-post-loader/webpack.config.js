// Migrated temporarily from the webpack's original test implementation.
// expect(require("./a")).toBe("resource loader2 loader1 loader3");
// expect(require("!./a")).toBe("resource loader2 loader3");
// expect(require("!!./a")).toBe("resource");
// expect(require("-!./a")).toBe("resource loader3");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	module: {
		rules: [
			// disable nothing
			{
				test: /a\.js$/,
				resourceQuery: /t0/,
				use: "./loader1"
			},
			{
				test: /a\.js$/,
				resourceQuery: /t0/,
				use: "./loader2",
				enforce: "pre"
			},
			{
				test: /a\.js$/,
				resourceQuery: /t0/,
				use: "./loader3",
				enforce: "post"
			},

			// disable normal
			{
				test: /a\.js$/,
				resourceQuery: /t1/,
				use: [] // disabled
			},
			{
				test: /a\.js$/,
				resourceQuery: /t1/,
				use: "./loader2",
				enforce: "pre"
			},
			{
				test: /a\.js$/,
				resourceQuery: /t1/,
				use: "./loader3",
				enforce: "post"
			},

			// disable normal post pre
			{
				test: /a\.js$/,
				resourceQuery: /t2/,
				use: [] // disabled
			},
			{
				test: /a\.js$/,
				resourceQuery: /t2/,
				enforce: "pre",
				use: [] // disabled
			},
			{
				test: /a\.js$/,
				resourceQuery: /t2/,
				enforce: "post",
				use: [] // disabled
			},

			// disable normal pre
			{
				test: /a\.js$/,
				resourceQuery: /t3/,
				use: [] // disabled
			},
			{
				test: /a\.js$/,
				resourceQuery: /t3/,
				enforce: "pre",
				use: [] // disabled
			},
			{
				test: /a\.js$/,
				resourceQuery: /t3/,
				use: "./loader3",
				enforce: "post"
			}
		]
	}
};
