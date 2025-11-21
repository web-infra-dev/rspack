it("should evaluate `typeof xxx` of `DefinePlugin`", () => {
	typeof window === "undefined" ? true : require("fail")
	expect(typeof window).toBe("undefined")
})
