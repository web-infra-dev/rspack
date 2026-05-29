import { createRequire as _createRequire } from "module";

it("should preserve created require resolve when requireResolve is disabled", () => {
	const require = _createRequire(import.meta.url);
	const resolved = require.resolve("./a");
	expect(resolved.endsWith("a.js")).toBe(true);
	expect(resolved).not.toBe("./a.js");

	const directResolved = _createRequire(import.meta.url).resolve("./a");
	expect(directResolved.endsWith("a.js")).toBe(true);
	expect(directResolved).not.toBe("./a.js");
});
