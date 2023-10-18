it("the result of loader content should be ordered", () => {
	expect(global.mockFn.mock.calls.length).toEqual(1);
});
