it("should resolve a static specifier to the emitted asset URL string", () => {
	const resolved = import.meta.resolve("./file.txt");
	expect(typeof resolved).toBe("string");
	expect(resolved).toContain("file.txt");
	expect(resolved).toBe(new URL("./file.txt", import.meta.url).href);
});

it("should resolve the same asset consistently", () => {
	const a = import.meta.resolve("./file.txt");
	const b = import.meta.resolve("./file.txt");
	expect(a).toBe(b);
});

it("should resolve a template literal specifier without expressions", () => {
	expect(import.meta.resolve(`./file.txt`)).toContain("file.txt");
});

it("should resolve computed member access", () => {
	expect(import.meta["resolve"]("./file.txt")).toContain("file.txt");
});

it("should resolve an ignored request to an empty data URL", () => {
  expect(import.meta.resolve("ignored")).toBe("data:,");
});

it("should preserve inline asset URLs", () => {
  expect(import.meta.resolve("data:text/plain,inline")).toBe(new URL("data:text/plain,inline", import.meta.url).href);
});
