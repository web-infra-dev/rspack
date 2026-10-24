const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("keeps a pruned sideEffects:false import out of the runtime chunk", () => {
	const runtime = fs.readFileSync(
		path.join(__dirname, "runtime-hoist_inactive_connections.js"),
		"utf-8"
	);

	expect(runtime).toContain("./node_modules/used-pkg/index.js");
	expect(runtime).not.toContain("pruned-pkg");
});
