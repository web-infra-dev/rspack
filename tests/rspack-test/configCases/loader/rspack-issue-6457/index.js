it("should share data between the pitch and the normal phase", () => {
	expect(require("!./loader.mjs!./loader2.mjs!./loader3.mjs!")).toStrictEqual({
		foo: "bar",
		bar: "baz"
	})
})
