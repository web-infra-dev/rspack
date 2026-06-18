import { createRequire } from "module";

// The argument below is a URL-constructor expression, not the deferrable bare
// import-meta-url form, so it is not preserved verbatim. With requireResolve disabled it
// must still get the normal argument replacement (baked to a build-time path); otherwise
// that expression would be left in this CommonJS bundle, which is a syntax error (the
// module would fail to even load). The forbidden substrings are checked from
// test.config.js so they are not part of this bundled source.
const r = createRequire(new URL(import.meta.url));

it("bakes a non-deferred createRequire URL argument so CommonJS output stays valid", () => {
	// The bundle loaded at all (no syntax error) and require() still bundles.
	expect(r("./dep")).toBe(1);
});
