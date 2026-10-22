rs.mockRequire("shared-format-mock");
rs.mockRequire("exact-format-mock");

it("should resolve a shared mock for a cjs entry", () => {
	expect(require("shared-format-mock").value).toBe("shared_mock");
});

it("should prefer the cjs mock over extension fallback", () => {
	expect(require("exact-format-mock").value).toBe("exact_require_mock");
});
