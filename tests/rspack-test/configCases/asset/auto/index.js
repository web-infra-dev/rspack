import LARGE from "./large.png";
import SMALL from "./logo.svg";

const RESOURCE_REGEX = /[^/.]+\.(svg|png)$/;

it("should determine the generation based on size if `rule.type` is sat to `asset`", () => {
	expect(RESOURCE_REGEX.test(LARGE)).toBeTruthy();
	expect(SMALL.startsWith("data:image/svg+xml;base64,")).toBeTruthy();
});
