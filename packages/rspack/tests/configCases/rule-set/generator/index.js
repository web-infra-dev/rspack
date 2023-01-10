import CUSTOM from "./logo.png?custom";
import NON_CUSTOM from "./logo.svg?non-custom";

it("should override the `output.assetModuleFilename` if `module.rule.generator.filename` is configured", () => {
	expect(CUSTOM.startsWith("/custom-asset")).toBeTruthy();
});

it("should use the `output.assetModuleFilename` if `module.rule.generator.filename` is not configured", () => {
	expect(NON_CUSTOM.startsWith("/asset")).toBeTruthy();
});
