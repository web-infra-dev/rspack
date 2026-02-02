/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		target: "node",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "node14.17",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "node14.18",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "node15",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "node16",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "browserslist:node 14.18.0, node 16.0.0",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "browserslist:node 14.18.0, node 15.0.0, node 16.0.0",
		experiments: {
			},
		output: {
			module: true,
			chunkFormat: "module"
		}
	},
	{
		target: "node"
	}
];
