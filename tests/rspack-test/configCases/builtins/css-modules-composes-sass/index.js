const path = require("path");

it("css modules in scss", () => {
	const style = require("./index.scss");
	expect(style).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'index.scss.txt'));
});
