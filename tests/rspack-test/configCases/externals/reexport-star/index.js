const fs = require("fs");
const path = require("path");
const readCase = (name) => fs.readFileSync(path.resolve(__dirname, `${name}.mjs`), "utf-8");
let snapshotDir;
if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
	snapshotDir = path.join(__SNAPSHOT__, "runtimeModeSnapshot");
} else {
	snapshotDir = __SNAPSHOT__;
}

it("reexport star from external module", function () {
	expect(readCase("case1")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case1.txt'));
	expect(readCase("case2")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case2.txt'));
	expect(readCase("case3")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case3.txt'));
	expect(readCase("case4")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case4.txt'));
	expect(readCase("case5")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case5.txt'));
	expect(readCase("case6")).toMatchFileSnapshotSync(path.join(snapshotDir, 'case6.txt'));
});
