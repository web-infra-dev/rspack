it("should not use `target` to control user code downgrading with `disableTransformByDefault` enabled", () => {
	const a = () => {};
	expect(a.toString()).toContain("=>");
});
