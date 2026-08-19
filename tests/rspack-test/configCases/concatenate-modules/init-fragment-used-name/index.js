import { __webpack_require__ as localRequire } from "./local";

it("should preserve runtime globals referenced by init fragments", () => {
	expect(localRequire()).toBe("captured");
	expect(PROVIDED_VALUE).toBe("provided");
});
