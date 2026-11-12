it("should externalize bare and prefixed Node builtins", () => {
	expect(require("fs")).toBe(require("node:fs"));
	expect(require("assert/strict")).toBe(require("node:assert/strict"));
});

it("should externalize unknown node-prefixed requests for forward compatibility", () => {
	expect(() => require("node:rspack-future-builtin")).toThrow();
});

it("should keep pnpapi external without resolving it at build time", () => {
	const loadPnp = () => require("pnpapi");
	expect(typeof loadPnp).toBe("function");
});

it("should only treat node: at the start of a request as external", () => {
	expect(require("./node:local")).toBe(42);
});
