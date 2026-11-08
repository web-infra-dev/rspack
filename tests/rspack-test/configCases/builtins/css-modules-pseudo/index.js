const path = require("path");

it("css modules pseudo syntax", () => {
	const style = require("./index.css");
	expect(style).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'index.css.txt'));
});
