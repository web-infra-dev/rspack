const element = <div></div>;

it("react automatic even with `disableTransformByDefault` is on", () => {
	expect(element.type).toBe("div");
});
