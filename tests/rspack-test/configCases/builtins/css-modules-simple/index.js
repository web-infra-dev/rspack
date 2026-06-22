const path = require("path");

it("css modules simple test", () => {
	const style = require("./index.module.css");
	expect(style).toMatchFileSnapshotSync(path.join(__SNAPSHOT__, 'index.module.css.txt'));
});
