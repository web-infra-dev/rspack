import path from "path";
import { test } from "uvu";
import assert from "assert";
import { Rspack } from "../src";

const context = path.resolve(__dirname, "../../../examples/react-with-sass");

const rspack = new Rspack({
	entry: {
		main: path.resolve(context, "./src/index.jsx")
	},
	context,
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

test("sass", async () => {
	const stats = await rspack.build();
	assert.equal(stats.errors.length, 0);
});

test.run();
