import fs from "fs";
import path from "path";
import img from "./img.png";

it("should return the correct size for asset", async () => {
	const statsSize = __STATS__.assets.find(a => a.name === img).size;
	const realSize = (await fs.promises.stat(path.resolve(__dirname, img))).size;
	expect(statsSize).toBe(realSize);
});
