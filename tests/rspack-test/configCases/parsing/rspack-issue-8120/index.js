const fs = __non_webpack_require__("node:fs");
const path = __non_webpack_require__("node:path");

const { NODE_ENV, PUBLIC_URL } = process.env.test;

it("should work", function () {
	let js = fs.readFileSync(path.resolve(__dirname, "./bundle0.js"), "utf-8");

	expect(
		js.includes(
			'const { NODE_ENV, PUBLIC_URL } = ({ "NODE_ENV":"development","PUBLIC_URL":"" });'
		)
	).toBeTruthy();
});
