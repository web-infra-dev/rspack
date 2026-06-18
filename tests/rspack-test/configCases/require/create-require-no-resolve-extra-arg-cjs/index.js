import { createRequire } from "module";

// A multi-argument createRequire is not deferrable (a clear would drop the extra argument's
// side effect). With requireResolve disabled in CommonJS output, the literal first argument
// must still be baked to a build-time path so nothing invalid leaks into the bundle, while
// the extra argument's side effect is preserved. The forbidden substring is checked from
// test.config.js so it is not part of this bundled source.
let sideEffectRan = false;
const r = createRequire(import.meta.url, (sideEffectRan = true));

it("bakes a non-deferred multi-arg createRequire in CommonJS, keeping the side effect", () => {
	// require() is still bundled and runs (the bundle loaded with no syntax error).
	expect(r("./dep")).toBe(1);
	// The extra argument's side effect still ran (the call was not cleared away).
	expect(sideEffectRan).toBe(true);
});
