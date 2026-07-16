import { createRequire } from "node:module";
import { bundledValue } from "./only-require.js";

const req = createRequire(import.meta.url);

export const resolved = req.resolve("path");

it("should preserve import.meta.url when requireResolve is disabled", async () => {
	const fs = await import(/* webpackIgnore: true */ "node:fs");
	const path = await import(/* webpackIgnore: true */ "node:path");
	const source = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");
	const runtimeCreateRequire =
		"(0,external_node_module_namespaceObject." + "createRequire)(import.meta.url)";

	expect(source.split(runtimeCreateRequire)).toHaveLength(2);
	expect(source).not.toContain("file:" + "//");
	expect(source).toContain("/* createRequire() */ undefined");
	expect(resolved).toBe("path");
	expect(bundledValue).toBe("dep");
});
