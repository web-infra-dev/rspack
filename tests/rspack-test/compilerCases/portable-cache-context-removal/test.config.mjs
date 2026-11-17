import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { rspack } from "@rspack/core";
import {
	closeCompiler,
	runCompiler
} from "@rspack/test-tools/helper/lifecycle";

async function build(context) {
	const compiler = rspack({
		context,
		mode: "development",
		devtool: false,
		entry: "./index.js",
		output: { path: path.join(context, "dist") },
		cache: { type: "persistent", portable: true }
	});
	try {
		const stats = await runCompiler(compiler);
		const json = stats.toJson({ all: false, errors: true, modules: true });
		expect(json.errors).toEqual([]);
		return json.modules.map(module => module.identifier);
	} finally {
		await closeCompiler(compiler);
	}
}

/** @type {import("@rspack/test-tools").TCompilerCaseConfig} */
export default {
	description:
		"should remove a context module from the portable cache when its regex contains the root",
	async run() {
		const base = fs.realpathSync.native(
			fs.mkdtempSync(path.join(os.tmpdir(), "rspack-portable-context-removal-"))
		);
		try {
			fs.mkdirSync(path.join(base, "dir"));
			fs.writeFileSync(path.join(base, "dir", "a.js"), "module.exports = 1;");
			// A space in the regex source is a token delimiter in identifiers, so the root that
			// follows it is converted differently with and without the identifier layout.
			const source = `x ${base}/dir/a`;
			fs.writeFileSync(
				path.join(base, "index.js"),
				`require.context("./dir", false, new RegExp(${JSON.stringify(source)}));`
			);
			expect(await build(base)).toHaveLength(2);

			fs.writeFileSync(path.join(base, "index.js"), "console.log(\"no context\");");
			expect(await build(base)).toHaveLength(1);
			// The removed context module must not come back from the cache.
			expect(await build(base)).toHaveLength(1);
		} finally {
			fs.rmSync(base, { recursive: true, force: true });
		}
	}
};
