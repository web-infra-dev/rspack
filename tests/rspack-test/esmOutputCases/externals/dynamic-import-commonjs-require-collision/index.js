const require = () => {
	throw new Error("user require should not be called");
};

import.meta.__customImport__ = async request => {
	throw new Error(`custom import should not load ${request}`);
};

export async function loadPlatform(require) {
	const os = await import("os");
	return [typeof os.platform, typeof os.default.platform, require];
}

it("should not capture user require for commonjs dynamic external", async () => {
	expect(await loadPlatform("local-require")).toEqual([
		"function",
		"function",
		"local-require"
	]);
});

export { require as userRequire };
