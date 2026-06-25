const path = require("path");

it("css modules localIdentName with hash", () => {
	const style = require("./index.css");
	expect(style).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'index.css.txt'));
});
