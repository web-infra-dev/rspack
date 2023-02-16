import path from "path";
import { Rspack } from "../src";
import postcssLoader from "@rspack/postcss-loader";

const context = path.resolve(__dirname, "../../../examples/react-with-less");

const rspack = new Rspack({
	entry: {
		main: path.resolve(context, "./src/index.jsx")
	},
	context,
	define: {
		"process.env.NODE_ENV": JSON.stringify("development")
	},
	builtins: {
		html: [{}]
	},
	module: {
		rules: [
			{
				test: /\.module\.less$/,
				use: [
					{ loader: "less-loader" },
					{
						loader: postcssLoader,
						options: {
							modules: true
						}
					}
				],
				type: "css"
			},
			{
				test: /\.less$/,
				use: [{ loader: lessLoader }],
				type: "css"
			}
		]
	}
});

async function main() {
	const stats = await rspack.build();
	console.log(stats);
}

main();
