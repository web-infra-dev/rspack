it("should allow transitive overrides (container-no-shared/a)", () => {
	return import("container-no-shared/a").then(({ value }) => {
		expect(value).toBe("new shared");
	});
});

it("should not override non-overridables (container-no-shared/b)", () => {
	return import("container-no-shared/b").then(({ value }) => {
		expect(value).toBe("shared");
	});
});

it("should have good module ids", async () => {
	const { default: m0 } = await import(
		"container-no-shared/modules-from-remote"
	);
	const { default: m1 } = await import("container-no-shared/modules");
	const m2 = Object.keys(__webpack_modules__).sort();
	[
		"./b.js",
		"./modules.js",
		"rspack/container/entry/container-with-shared",
		"rspack/sharing/consume/default/shared/./shared"
	].forEach(m => expect(m0).toContain(m));
	[
		"./a.js",
		"./b.js",
		"./modules-from-remote.js",
		"./modules.js",
		"./shared.js",
		"rspack/container/entry/container-no-shared",
		"rspack/container/reference/container-with-shared",
		"rspack/container/remote/container-with-shared/b",
		"rspack/container/remote/container-with-shared/modules"
	].forEach(m => expect(m1).toContain(m));
	[
		"./index.js",
		"./shared.js",
		"rspack/container/reference/container-no-shared",
		"rspack/container/remote/container-no-shared/a",
		"rspack/container/remote/container-no-shared/b",
		"rspack/container/remote/container-no-shared/modules",
		"rspack/container/remote/container-no-shared/modules-from-remote"
	].forEach(m => expect(m2).toContain(m));
});
