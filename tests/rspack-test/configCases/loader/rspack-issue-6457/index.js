it("should share data between the pitch and the normal phase", () => {
	expect(require("!./loader!./loader2!./loader3!")).toStrictEqual({
		foo: "bar",
		bar: "baz"
	})
})
