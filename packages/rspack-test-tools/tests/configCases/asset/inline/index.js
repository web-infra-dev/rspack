import IMG from "./img.png";

it("should inline the source if `rule.type` is sat to `asset/inline`", () => {
	expect(IMG.startsWith("data:image/png;base64,")).toBeTruthy();
});
