import { a } from "./pkg";

// Rspack keeps modules built by earlier rebuilds in the graph, so check what is bundled.
const bundled = (file) =>
	__STATS__.modules.some((m) => m.name.endsWith(file) && m.chunks.length > 0);

it("should defer the import declaration again once only a re-export is requested", () => {
	expect(a).toBe("a");
	expect(bundled("pkg/x.js")).toBe(false);
});
