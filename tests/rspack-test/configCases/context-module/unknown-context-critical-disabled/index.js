it("should import context module", async () => {
	try {
		require(["unknown/a"], function (a) {
			expect(a.default).toBe("a")
		})
	} catch (e) {
	}
})
