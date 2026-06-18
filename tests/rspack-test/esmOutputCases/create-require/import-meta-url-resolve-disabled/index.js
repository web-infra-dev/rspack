import { createRequire } from "node:module";

const req = createRequire(import.meta.url);

// `require()` is still bundled; `require.resolve` is preserved for the runtime
// because module.parser.javascript.requireResolve is disabled.
export const value = req("./dep.js");
export const resolved = req.resolve("path");
export const inlineResolved = createRequire(import.meta.url).resolve("path");

// A created require used only via the 2-arg `require.resolve(request, options)` form must
// still be kept (regression: the multi-argument resolve must mark the declaration used).
const reqWithOptions = createRequire(import.meta.url);
export const resolvedWithOptions = reqWithOptions.resolve("path", { paths: [] });

// An extra createRequire argument with a side effect must be preserved (such a call is not
// deferred, so a clear cannot drop the side effect).
let sideEffectRan = false;
const reqExtraArg = createRequire(import.meta.url, (sideEffectRan = true));
export const extraArgValue = reqExtraArg("./dep.js");

// A multi-argument INLINE createRequire member chain keeps its literal `import.meta.url`
// (NOT baked to a build-time URL — guarded by the file-url-scheme assertion below) and
// preserves the extra argument's side effect.
let inlineMultiArgRan = false;
export const inlineMultiArgResolved = createRequire(
	import.meta.url,
	(inlineMultiArgRan = true)
).resolve("path");

it("should preserve createRequire().resolve at runtime when requireResolve is disabled", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const source = fs.readFileSync(path.join(__dirname, "main.mjs"), "utf-8");

	// Built by concatenation so these markers don't appear in this file's own
	// source, which is itself emitted into main.mjs (see import-meta-url-resolve).
	const clearedMarker = "/* createRequire()" + " */ undefined";
	const fileUrlScheme = "file:" + "//";
	const resolveRewrite = "/*require" + ".resolve*/";

	// The createRequire(...) call is kept with import.meta.url preserved verbatim:
	// NOT cleared to `undefined`, and NOT baked to a build-time absolute file URL.
	expect(source).toContain("createRequire(import.meta.url)");
	expect(source).not.toContain(clearedMarker);
	expect(source).not.toContain(fileUrlScheme);
	// require() still routes through the bundler; require.resolve stays a runtime call.
	expect(source).toContain("__webpack_require__(");
	expect(source).toContain('.resolve("path")');
	expect(source).not.toContain(resolveRewrite);

	// require() is bundled.
	expect(value).toBe("dep");
	// require.resolve runs at runtime: a builtin resolves to its own name, which a
	// build-time module-id rewrite could never produce.
	expect(resolved).toBe("path");
	expect(inlineResolved).toBe("path");
	// A real runtime resolve throws for a missing module instead of returning an id.
	expect(() => req.resolve("./does-not-exist")).toThrow();

	// P1: the 2-arg resolve form kept its created require (a cleared `undefined` would throw).
	expect(resolvedWithOptions).toBe("path");
	// P2: the extra-argument createRequire kept its side effect, and require() still bundles.
	expect(extraArgValue).toBe("dep");
	expect(sideEffectRan).toBe(true);

	// Multi-argument inline member chain: `.resolve` runs at runtime (kept, not `undefined`),
	// the extra argument's side effect ran, and `import.meta.url` was kept literal (the
	// file-url-scheme assertion above guards against baking it to a build-time path).
	expect(inlineMultiArgResolved).toBe("path");
	expect(inlineMultiArgRan).toBe(true);
});
