it("should pass only the exported property to jsonp callback", function () {
	expect(global.__jsonpExportCapture).toBe(42);
});

export const answer = 42;
export const other = "not exported";
