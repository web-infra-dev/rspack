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
	expect(
		fs.readFileSync(path.join(__dirname, "./bundle0.js"), "utf-8")
	).toContain("__webpack_require__." + name + " = ");
});
