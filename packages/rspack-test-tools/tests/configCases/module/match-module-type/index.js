import INLINE_SVG from "./img.svg?inline";
import RESOURCE_SVG from "./img.svg";

import RESOURCE_PNG from "./img.png?inline";
import RESOURCE_PNG_2 from "./img.png";

// FIXME: We should align this with target `Node`, currently the `__webpack_require__.p` is not defined for the `Node`.
const RESOURCE_REGEX = /\.(svg|png)/;

it("should use the last matching type if it is matched with multiple module rules", () => {
	expect(INLINE_SVG.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
	expect(RESOURCE_REGEX.test(RESOURCE_PNG)).toBeTruthy();
	expect(RESOURCE_REGEX.test(RESOURCE_PNG_2)).toBeTruthy();
});

it("should use the matching type if only a single rule matches", () => {
	expect(RESOURCE_REGEX.test(RESOURCE_SVG)).toBeTruthy();
});
