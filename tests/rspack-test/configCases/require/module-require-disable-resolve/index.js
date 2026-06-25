import { createRequire as _createRequire } from "module";
import fs from "fs";
import path from "path";

it("should preserve created require resolve when requireResolve is disabled", () => {
	const require = _createRequire(import.meta.url);
	const resolved = require.resolve("./a");
	expect(resolved.endsWith("a.js")).toBe(true);
	expect(resolved).not.toBe("./a.js");

	const directResolved = _createRequire(import.meta.url).resolve("./a");
	expect(directResolved.endsWith("a.js")).toBe(true);
	expect(directResolved).not.toBe("./a.js");

	const directResolvedWithUrl = _createRequire(
		new URL("./foo/c.js", import.meta.url)
	).resolve("./a");
	expect(directResolvedWithUrl).toMatch(/[\\/]foo[\\/]a\.js$/);
	expect(directResolvedWithUrl).not.toBe("./a.js");
});

it("should keep preserved createRequire argument dependencies", () => {
	try {
		_createRequire(import("./async-context")).resolve("./a", {});
	} catch {}

	let dynamicUrlBaseExtraEvaluated = false;
	try {
		_createRequire(
			new URL(
				"./foo/c.js",
				import("./async-url-context"),
				(dynamicUrlBaseExtraEvaluated = true)
			)
		).resolve("./a", {});
	} catch {}
	expect(dynamicUrlBaseExtraEvaluated).toBe(true);

	const emittedSource = fs
		.readdirSync(path.dirname(__filename))
		.filter(file => file.endsWith(".js"))
		.map(file => fs.readFileSync(path.join(path.dirname(__filename), file), "utf-8"))
		.join("\n");
	expect(
		emittedSource.includes("__rspackCreateRequireUnsupportedResolveContextDependency")
	).toBe(true);
	expect(
		emittedSource.includes("__rspackCreateRequireUnsupportedResolveUrlDependency")
	).toBe(true);
});
