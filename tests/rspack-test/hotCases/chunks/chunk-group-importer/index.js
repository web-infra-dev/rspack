import { load, loadContext, loadLate } from "./loader";

it("keeps importers unchanged when their group gains a split chunk", async () => {
	expect((await load()).default).toBe(1);
	expect((await loadContext("page")).default).toBe(1);
	await NEXT_HMR();
	// Neither this entry nor loader.js accepts updates. Only page.js should run.
	expect(globalThis.chunkGroupImporterValue).toBe(2);
	expect((await load()).default).toBe(2);
	expect((await loadContext("page")).default).toBe(2);
	// This group was not loaded before the update and needs the new mapping too.
	expect((await loadLate()).default).toBe(2);
});
