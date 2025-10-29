import IMG from "./logo.svg";

const RESOURCE_REGEX = /[^/.]+\.svg$/;

it("should externalize the source if `rule.type` is sat to `asset/resource`", () => {
	expect(RESOURCE_REGEX.test(IMG)).toBeTruthy();
});
