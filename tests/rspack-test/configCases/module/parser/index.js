import RESOURCE_PNG from "./logo.png?should-be-externalized";
import INLINE_PNG from "./logo.png?should-be-inlined";

const RESOURCE_REGEX = /\.png/;

it("should override the `module.parser.assets.dataUrlCondition.maxSize` if `module.rule.parser.dataUrlCondition.maxSize` is configured", () => {
	expect(INLINE_PNG.startsWith("data:image/png;base64,")).toBeTruthy();
});

it("should use `module.parser.assets.dataUrlCondition.maxSize` if `module.rule.parser.dataUrlCondition.maxSize` is not configured", () => {
	expect(RESOURCE_REGEX.test(RESOURCE_PNG)).toBeTruthy();
});
