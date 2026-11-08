import "./link-root.link.css";

const STATS = __STATS__.children[__STATS_I__];

it("should emit link-export css", () => {
	const fs = require("fs");
	const path = require("path");
	const css = fs.readFileSync(
		path.join(STATS.outputPath, `bundle${__STATS_I__}.css`),
		"utf-8"
	);
	expect(css).toContain(".link-root");
	expect(css).toContain(".link-leaf");
});
