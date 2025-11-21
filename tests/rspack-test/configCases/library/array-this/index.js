it("should define property in 'window' object", function () {
	expect(this["a"]["b"]).toBeDefined();
});
