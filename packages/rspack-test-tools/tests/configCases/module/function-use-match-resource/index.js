import { A, B, __info__, __context__ } from "__label__?A=A#B=B!=!./lib";

it("use entry arguments should be correct", () => {
	expect(A).toBe('A');
	expect(B).toBe('B');
	expect(__info__.resource).toBe("__label__");
	expect(__info__.realResource.includes("lib.js")).toBe(true);
	expect(__info__.resourceQuery).toBe("?A=A");
	expect(__info__.resourceFragment).toBe("#B=B");
	expect(__info__.issuer.includes("index.js")).toBe(true);
	expect(__info__.issuerLayer).toBe("");
});
