import path from "path";
import rspack from "@rspack/core";
declare const enum JSB {
	SECRET = 42
}
export default {
	mode: "production",
	entry: path.resolve(__dirname, "main.ts"),

	output: {
		path: path.resolve(__dirname, "dist"),
		filename: "node-register.bundle.js"
	},
	plugins: [
		new rspack.DefinePlugin({
			ANSWER: JSB.SECRET
		})
	]
};
