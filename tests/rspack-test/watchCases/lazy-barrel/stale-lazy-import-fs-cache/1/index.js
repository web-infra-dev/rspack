import { foo } from "./pkg";

// Rspack keeps modules built by earlier rebuilds in the graph, so check what is bundled.
const bundled = (file) =>
	__STATS__.modules.some((m) => m.name.endsWith(file) && m.chunks.length > 0);

// pkg is not rebuilt, so the previous build's lazy state must not carry over
it("should build the import a newly requested local export needs", () => {
	expect(foo).toBe("x");
	expect(bundled("pkg/x.js")).toBe(true);
});

// webpack also expects pkg/a.js to be lazy again here. Rspack keeps re-exports
// activated by an earlier rebuild, so a.js stays in the graph.
