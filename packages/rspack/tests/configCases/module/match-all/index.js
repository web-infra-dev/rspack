import INLINE from "./logo.svg?inline";
import RESOURCE from "./logo.svg";

const RESOURCE_REGEX = /^\/assets\/[^/.]+\.svg$/;

it("should generate urls based on loader options", () => {
	expect(INLINE.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
	expect(RESOURCE_REGEX.test(RESOURCE)).toBeTruthy();
});
