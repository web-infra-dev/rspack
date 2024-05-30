import source from "./a.js!=!data:javascript,var b, c;export const a = (b ?? (c ??= 2 ** 2))";

it("should transformed to es3 snytax", () => {
	expect(source.includes("??")).toBe(false)
	expect(source.includes("??=")).toBe(false)
	expect(source.includes("**")).toBe(false)
});
