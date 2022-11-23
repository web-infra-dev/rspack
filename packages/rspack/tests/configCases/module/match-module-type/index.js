import INLINE_SVG from "./large.svg?inline";
import RESOURCE_SVG from "./logo.svg";

import RESOURCE_PNG from "./large.png?inline";
import RESOURCE_PNG_2 from "./logo.png";

const RESOURCE_REGEX = /^\/assets\/[^/.]+\.svg$/;

it("should use the last matching type if it is matched with multiple module rules", () => {
	expect(INLINE_SVG.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
	expect(RESOURCE_REGEX.test(RESOURCE_PNG)).toBeTruthy();
	expect(RESOURCE_REGEX.test(RESOURCE_PNG_2)).toBeTruthy();
});

it("should use the matching type if only a single rule matches", () => {
	expect(RESOURCE_REGEX.test(RESOURCE_SVG)).toBeTruthy();
});
