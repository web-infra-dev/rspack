const fs = require("fs");

it("processAssetsStageAdditions stage and update asset should works", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(mainFile.startsWith("/** MMMMM */")).toBeTruthy();
});
