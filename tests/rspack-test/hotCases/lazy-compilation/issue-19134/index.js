const path = require("path");

it("first activation of a lazy 'page' must not throw under UMD + closure externals (issue #9023)", async () => {
	let resolved;
	const promise = import("./page").then(r => (resolved = r));
	expect(resolved).toBe(undefined);
	await NEXT_HMR_DEFERRED();
	const result = await promise;
	expect(result).toHaveProperty("isFile");
	expect(result).toHaveProperty("joinedPath");
	expect(typeof result.isFile).toBe("function");
	expect(result.joinedPath).toBe(path.join("a", "b"));
});
