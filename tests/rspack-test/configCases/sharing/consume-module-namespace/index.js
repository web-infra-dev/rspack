function checkNamespaces({ values }) {
	const { cjsDefault, cjsNamespace, greet, esmDefault, esmNamespace, named } = values;
	expect(cjsDefault.greet()).toBe("hello");
	expect(greet()).toBe("hello");
	expect(cjsNamespace.default).toBe(cjsDefault);
	expect(cjsNamespace.default.greet()).toBe("hello");
	expect(cjsNamespace.greet).toBe(greet);
	expect(cjsNamespace.__esModule).toBe(true);

	expect(esmDefault()).toBe("esm default");
	expect(named).toBe("esm named");
	expect(esmNamespace.default).toBe(esmDefault);
	expect(esmNamespace.named).toBe(named);
	expect(esmNamespace.__esModule).toBe(true);
}

it("should preserve shared module namespaces in .js consumers", async () => {
	checkNamespaces(await import("./consumer.js"));
});

it("should preserve shared module namespaces in .mjs consumers", async () => {
	checkNamespaces(await import("./consumer.mjs"));
});
