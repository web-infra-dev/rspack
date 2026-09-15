async function loadModule(name) {
	return import("./dir/" + name);
}

async function loadModuleWithExclude(name) {
	return import(/* webpackExclude: /module-b\.js$/ */ "./dir/" + name);
}

async function loadModuleWithRspackExclude(name) {
	return import(/* rspackExclude: /module-b\.js$/ */ "./dir/" + name);
}

async function loadModuleWithInclude(name) {
	return import(/* webpackInclude: /module-b\.js$/ */ "./dir/" + name);
}

async function loadModuleWithRspackInclude(name) {
	return import(/* rspackInclude: /module-b\.js$/ */ "./dir/" + name);
}

async function loadModuleWithMode(name) {
	return import(/* webpackMode: "eager" */ "./dir/" + name);
}

async function loadModuleWithRspackMode(name) {
	return import(/* rspackMode: "eager" */ "./dir/" + name);
}

it("should work when no options", async () => {
	expect((await loadModule("module-a.js")).default).toBe("a");
	expect((await loadModule("module-b.js")).default).toBe("b");
	expect((await loadModule("module-c.js")).default).toBe("c");
});

it("should work with exclude", async () => {
	expect((await loadModuleWithExclude("module-a.js")).default).toBe("a");
	await expect(loadModuleWithExclude("module-b.js")).rejects.toThrow("Cannot find module './module-b.js'");
	expect((await loadModuleWithExclude("module-c.js")).default).toBe("c");
});

it("should work with rspack exclude", async () => {
	expect((await loadModuleWithRspackExclude("module-a.js")).default).toBe("a");
	await expect(loadModuleWithRspackExclude("module-b.js")).rejects.toThrow("Cannot find module './module-b.js'");
	expect((await loadModuleWithRspackExclude("module-c.js")).default).toBe("c");
});

it("should work with include", async () => {
	await expect(loadModuleWithInclude("module-a.js")).rejects.toThrow("Cannot find module './module-a.js'");
	expect((await loadModuleWithInclude("module-b.js")).default).toBe("b");
	await expect(loadModuleWithInclude("module-c.js")).rejects.toThrow("Cannot find module './module-c.js'");
});

it("should work with rspack include", async () => {
	await expect(loadModuleWithRspackInclude("module-a.js")).rejects.toThrow("Cannot find module './module-a.js'");
	expect((await loadModuleWithRspackInclude("module-b.js")).default).toBe("b");
	await expect(loadModuleWithRspackInclude("module-c.js")).rejects.toThrow("Cannot find module './module-c.js'");
});

it("should work with mode", async () => {
	expect((await loadModuleWithMode("module-a.js")).default).toBe("a");
	expect((await loadModuleWithMode("module-b.js")).default).toBe("b");
	expect((await loadModuleWithMode("module-c.js")).default).toBe("c");
});

it("should work with rspack mode", async () => {
	expect((await loadModuleWithRspackMode("module-a.js")).default).toBe("a");
	expect((await loadModuleWithRspackMode("module-b.js")).default).toBe("b");
	expect((await loadModuleWithRspackMode("module-c.js")).default).toBe("c");
});
