import "./index.css";
const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should modify runtime module source in main", () => {
	const name = "APP_ROOT";
	expect(
		fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8")
	).toContain("globalThis." + name);
});

it("should has css loading hmr runtime requirements", () => {
	const name = "hmrC.css";
	const source = fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8");
	if (globalThis.__RSPACK_TEST_RUNTIME_MODE_RSPACK) {
		expect(source).toContain("__rspack_hmrDownloadUpdateHandlers.css = ");
	} else {
		expect(source).toContain("__webpack_require__." + name + " = ");
	}
});
