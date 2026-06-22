const path = require("path");

it("css modules dedupe", () => {
	const style = require("./source.css");
	expect(style).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'source.css.txt'));
});
