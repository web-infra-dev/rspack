import INLINE from "./logo.svg?inline";
import RESOURCE from "./logo.svg";

const RESOURCE_REGEX = /^\/assets\/[^/.]+\.svg$/;

it("should generate inline svg if both `test` and `resourceQuery` matches", () => {
	expect(INLINE.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
});

it("should generate svg as a resource if only `test` matches", () => {
	expect(RESOURCE_REGEX.test(RESOURCE)).toBeTruthy();
});
