import { createRequire } from "module";

it("should preserve context dependencies when created require escapes", () => {
	const escapedRequire = createRequire(new URL("./escaped/c.js", import.meta.url));
	function register(value) {
		globalThis.__rspackEscapedRequire = value;
	}
	register(escapedRequire);
	expect(typeof globalThis.__rspackEscapedRequire).toBe("function");
});
