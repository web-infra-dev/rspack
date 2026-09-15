export async function loadPlatform() {
	const os = await import("os");
	return [os.platform, os.default.platform];
}

it("should wrap commonjs dynamic external as a namespace object", async () => {
	const [platform, defaultPlatform] = await loadPlatform();

	expect(platform).toBe(defaultPlatform);
});
