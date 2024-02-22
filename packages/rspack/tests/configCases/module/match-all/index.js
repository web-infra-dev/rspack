import INLINE from "./img.svg?inline";

it("should generate inline svg if both `test` and `resourceQuery` matches", () => {
	expect(INLINE.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
});
