import x from "x";
import y from "y";

it("should work", () => {
	expect(x).toBe(42);
	expect(y).toBe(24);
})

it("should add provided modules to the share scope - no matter the resolver", async () => {
	await __webpack_init_sharing__("default");
	expect(Object.keys(__webpack_share_scopes__.default)).toContain("x");
	expect(Object.keys(__webpack_share_scopes__.default)).toContain("y");
});
