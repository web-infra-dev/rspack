import IMG from "./img.png";

it("should support function in dataUrlCondition", () => {
	expect(IMG.startsWith("data:image/png;base64,")).toBeTruthy();
});
