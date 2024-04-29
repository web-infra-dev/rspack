function loadLocale(name) {
	var local, aliasedRequire;
	aliasedRequire = require;
	local = aliasedRequire("./locale/" + name);
	return local;
}

it("alias require should be works", () => {
	expect(loadLocale("a")).toBe("a");
	expect(loadLocale("a.js")).toBe("a");
	expect(loadLocale("b")).toBe("b");
	expect(loadLocale("b.js")).toBe("b");
});
