it("should not use `target` to control user code downgrading", () => {
	const a = () => {};
	expect(a.toString()).toContain("=>");
});
