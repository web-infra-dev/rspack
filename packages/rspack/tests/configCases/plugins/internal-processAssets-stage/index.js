const fs = require("fs");

it("plugin", () => {
	const content = fs.readFileSync(__filename, "utf-8");
	expect(content).toMatch(
		`//PROCESS_ASSETS_STAGE_REPORT;
//PROCESS_ASSETS_STAGE_SUMMARIZE;
//PROCESS_ASSETS_STAGE_OPTIMIZE_INLINE;
//PROCESS_ASSETS_STAGE_NONE;
//PROCESS_ASSETS_STAGE_PRE_PROCESS;
//PROCESS_ASSETS_STAGE_ADDITIONAL;
`
	);
});
