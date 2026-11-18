it("should cache virtual files by content while respecting path rules", async () => {
	const value = COMPILER_INDEX < 2 ? "before" : "after";
	const managedValue = COMPILER_INDEX < 4 ? "before" : "after";
	const hasAdded = COMPILER_INDEX === 4 || COMPILER_INDEX === 5;
	expect(require("./value.js")).toBe(value);
	expect(require("./immutable/value.js")).toBe("before");
	expect(require("virtual-cache-package/value.js")).toBe(managedValue);
	expect(require("virtual-cache-package/unmanaged/value.js")).toBe(value);

	// Content changes alone must not invalidate immutable or managed paths.
	const immutable = require.context("./immutable", false, /\.js$/);
	expect(immutable("./value.js")).toBe("before");
	const managed = require.context(
		"./node_modules/virtual-cache-package",
		false,
		/\.js$/
	);
	expect(managed("./value.js")).toBe(managedValue);
	const unmanaged = require.context(
		"./node_modules/virtual-cache-package/unmanaged",
		false,
		/\.js$/
	);
	expect(unmanaged.keys().sort()).toEqual(
		hasAdded
			? ["./added.js", "./real.js", "./value.js"]
			: ["./real.js", "./value.js"]
	);
	expect(unmanaged("./real.js")).toBe("real");
	expect(unmanaged("./value.js")).toBe(value);

	// Exercise missing -> resolved -> missing resolver snapshots as well.
	let added = "missing";
	try {
		added = require("virtual-cache-package/unmanaged/added.js");
	} catch (error) {
		expect(error.code).toBe("MODULE_NOT_FOUND");
	}
	expect(added).toBe(hasAdded ? "added" : "missing");
	if (hasAdded) expect(unmanaged("./added.js")).toBe("added");

	const transition = require.context("./transition", false, /\.txt$/);
	expect(transition.keys()).toEqual(["./value.txt"]);
	expect(transition("./value.txt")).toBe(
		COMPILER_INDEX === 2 || COMPILER_INDEX === 3 ? "f946684800000" : "real"
	);

	// Baseline, content change, addition and removal each get an unchanged restart.
	if (COMPILER_INDEX < 7) await NEXT_START();
});
