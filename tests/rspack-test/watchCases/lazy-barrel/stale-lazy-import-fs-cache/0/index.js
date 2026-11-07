import { a } from "./pkg";

// Rspack keeps modules built by earlier rebuilds in the graph, so check what is bundled.
const bundled = (file) =>
	__STATS__.modules.some((m) => m.name.endsWith(file) && m.chunks.length > 0);

// only a re-export is requested, so pkg's own `import { x }` is deferred too
it("should defer the barrel's import declaration initially", () => {
	expect(a).toBe("a");
	expect(bundled("pkg/x.js")).toBe(false);
});
