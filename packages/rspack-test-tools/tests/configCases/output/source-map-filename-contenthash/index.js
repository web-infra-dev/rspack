import fs from "fs/promises";
import("./chunk");

it("source-map-filename-contenthash should have correct name", async function () {
	const maps = (await fs.readdir(__dirname)).filter(i => i.endsWith(".map"));
	maps.sort();
	expect(maps.length).toBe(2);
	expect(maps.every(m => {
		let name = m.replace(".js.map", "").split("-");
		return name[0] !== name[1];
	})).toBeTruthy();
});
