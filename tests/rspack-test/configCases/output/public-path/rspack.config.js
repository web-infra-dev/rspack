/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		output: {
			publicPath: "/static/[hash:11]/"
		}
	},
	{
		output: {
			publicPath: "/static/[fullhash:11]/"
		}
	},
	{
		output: {
			publicPath: () => "/static/[hash:11]/"
		}
	},
	{
		output: {
			publicPath: () => "/static/[fullhash:11]/"
		}
	},
	{
		output: {
			publicPath: ({ hash }) => {
				return `/static/${hash.slice(0, 11)}/`;
			}
		}
	}
].map(v => ({ mode: "development", ...v }));
