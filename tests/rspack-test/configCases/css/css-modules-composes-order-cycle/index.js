import * as cycleStyles from "./cycle-entry.module.css";

it("keeps cyclic composed css modules in source order", () => {
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");
	const css = fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");

	expect(cycleStyles).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, "exports.txt")
	);
	expect(css).toMatchFileSnapshotSync(
		path.join(__SNAPSHOT__, "bundle0.css.txt")
	);
});
