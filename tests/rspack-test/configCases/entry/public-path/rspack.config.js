const cases = [
	{
		import: "./a.js",
		publicPath: "/static/[hash:11]/"
	},
	{
		import: "./a.js",
		publicPath: "/static/[fullhash:11]/"
	},
	{
		import: "./a.js",
		publicPath: () => "/static/[hash:11]/"
	},
	{
		import: "./a.js",
		publicPath: () => "/static/[fullhash:11]/"
	},
	{
		import: "./a.js",
		publicPath: ({ hash }) => {
			return `/static/${hash.slice(0, 11)}/`;
		}
	}
];
let bundleId = 1;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: cases.reduce((acc, c) => {
		acc[`bundle${bundleId++}`] = { ...c };
		return acc;
	}, {}),
	plugins: [
		{
			apply(compiler) {
				const { EntryPlugin } = compiler.webpack;
				for (const c of cases) {
					new EntryPlugin(compiler.context, c.import, {
						name: `bundle${bundleId++}`,
						publicPath: c.publicPath
					}).apply(compiler);
				}
			}
		}
	],
	output: {
		filename: "[name].js"
	}
};
