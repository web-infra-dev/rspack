import { __webpack_require__ as localRequire } from "./local";
import assetUrl from "./asset.txt";

it("should preserve runtime globals referenced by generated asset modules", () => {
	expect(localRequire()).toBe("local");
	expect(assetUrl).not.toContain("captured/");
});
