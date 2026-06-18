import { createRequire } from "module";

it("does not leak import.meta.url when createRequire is used only for require()", () => {
	// require.resolve is disabled, but this created require is only used for a plain
	// require(), never `.resolve`. So the createRequire call is not needed at runtime
	// and is cleared to `undefined` (its `module` import dropped) instead of leaving a
	// literal `import.meta.url`. That keeps the CommonJS artifact valid (a literal
	// `import.meta.url` would be a syntax error) and emits no warning. The require()
	// itself is still bundled.
	const require = createRequire(import.meta.url);
	expect(require("./dep")).toBe(1);
});
