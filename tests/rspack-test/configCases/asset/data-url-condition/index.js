import IMG from "./img.png";

it("should inline the content if `rule.type` is sat to `asset` and the size of the asset doesn't exceeds the `dataUrlCondition.maxSize`", () => {
	expect(IMG.startsWith("data:image/png;base64,")).toBeTruthy();
});
