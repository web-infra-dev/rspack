import path from "path";
import { Rspack } from "../src";

const context = path.resolve(__dirname, "../../../examples/react-with-sass");

const rspack = new Rspack({
	entry: {
		main: path.resolve(context, "./src/index.jsx")
	},
	context,
	plugins: ["html"],
	module: {
		rules: [
			{
				test: "\\.s[ac]ss$",
				uses: [{ builtinLoader: "sass-loader" }],
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
