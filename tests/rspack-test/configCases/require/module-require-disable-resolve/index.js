// The runtime-resolution behaviour of a preserved createRequire().resolve (with
// requireResolve disabled) is covered by the ESM case
// esmOutputCases/create-require/import-meta-url-resolve-disabled, because the
// preserved literal `import.meta.url` is only valid in ESM output. This CJS case
// covers that non-statically-analyzable createRequire arguments still keep their
// dependencies (dynamic import / new URL side effects) when resolve is disabled.
import { createRequire as _createRequire } from "module";
import fs from "fs";
import path from "path";

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
