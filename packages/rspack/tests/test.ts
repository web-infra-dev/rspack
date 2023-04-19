import { rspack } from "../dist";

rspack(
	{
		entry: {
			main: {
				import: "./fixtures/a.js"
			}
		},
		output: {
			filename: require("path").resolve(__dirname, "./dist/aa")
		},
		context: __dirname,
		optimization: {
			minimize: false
		},
		module: {
			rules: [
				{
					test: /\.js$/,
					use: [
						{
							loader: require.resolve("./tmp-loader/loader-a.js"),
							options: "heihei"
						},
						{
							loader: require.resolve("./tmp-loader/loader-mid.js"),
							options: "test"
						},
						{
							loader: require.resolve("./tmp-loader/loader-b.js"),
							options: {
								getItems() {
									return [1, 2, 3];
								}
							}
						},
						{
							loader: require.resolve("./tmp-loader/loader-c.js"),
							options: {
								getItems() {
									return [1, 2, 3];
								}
							}
						},
						{
							loader: require.resolve("./tmp-loader/loader-d.js"),
							options: {
								getItems() {
									return [1, 2, 3];
								}
							}
						}
					]
				}
			]
		}
	},
	(err, stats) => {
		console.log(err, stats?.toString());
	}
);
