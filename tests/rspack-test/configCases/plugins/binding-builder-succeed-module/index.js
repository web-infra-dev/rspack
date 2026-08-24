import { A, B } from "pkg";

it("should reroute individual ESM specifiers from a native plugin", () => {
	expect(A).toBe("A from shim");
	expect(B).toBe("B from shim");
});
