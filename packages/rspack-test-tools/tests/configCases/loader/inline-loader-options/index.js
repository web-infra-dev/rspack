it("should get the correct loader options", () => {
	expect(require("!!builtin:swc-loader??ruleSet[1].rules[0].use[0]!./lib.tsx").b).toBe("b")
})
