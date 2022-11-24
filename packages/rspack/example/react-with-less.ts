import path from "path";
import { Rspack } from "../src";
import lessLoader from "@rspack/plugin-less";
import postcssLoader from "@rspack/plugin-postcss";

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
				uses: [
					{ loader: lessLoader },
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
				uses: [{ loader: lessLoader }],
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
