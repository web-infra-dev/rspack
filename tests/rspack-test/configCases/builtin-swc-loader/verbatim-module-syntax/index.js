it("should handle type imports with verbatimModuleSyntax enabled by default", () => {
	const { getDefaultUser, defaultRole } = require("./lib");
	const user = getDefaultUser();
	expect(user.name).toBe("test");
	expect(user.age).toBe(18);
	expect(defaultRole).toBe("user");
});

