import fs from "fs";
import path from "path";

it("should include the correct split chunk ids in entry", async () => {
	if (Math.random() < 0) import("./module");
	const runtimeId = __STATS__.chunks.find(c => c.names.includes("runtime")).id;
	const entryCode = fs.readFileSync(
		path.resolve(__dirname, "entry.js"),
		"utf-8"
	);
	STATE.allIds = new Set([
		...(STATE.allIds || []),
		...__STATS__.entrypoints.entry.chunks
	]);
	const expectedIds = Array.from(STATE.allIds).filter(
		id => __STATS__.entrypoints.entry.chunks.includes(id) && id !== runtimeId
	);
	try {
		for (const id of STATE.allIds) {
			const expected = expectedIds.includes(id);
			const idStr = String(id);
			const isNumeric = /^\d+$/.test(idStr);
			// Match chunk ID in arrays like [681,834] or in .e(681) calls.
			const idPattern = isNumeric
				? new RegExp(`(?:[\\[,]${idStr}[\\],]|\\b\\.e\\(${idStr}\\))`)
				: new RegExp(`[\\[,]"${idStr}"[\\],]`);
			(expected ? expect(entryCode) : expect(entryCode).not).toMatch(idPattern);
		}
	} catch (e) {
		throw new Error(
			`Entrypoint code should contain only these chunk ids: ${expectedIds.join(
				", "
			)}\n${e.message}`
		);
	}
});
