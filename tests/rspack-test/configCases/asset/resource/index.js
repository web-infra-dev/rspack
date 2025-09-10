import IMG from "./logo.svg";

// FIXME: We should align this with target `Node`, currently the `__webpack_require__.p` is not defined for the `Node`. cc @underfin
const RESOURCE_REGEX = /[^/.]+\.svg$/;

it("should externalize the source if `rule.type` is sat to `asset/resource`", () => {
	expect(RESOURCE_REGEX.test(IMG)).toBeTruthy();
});
