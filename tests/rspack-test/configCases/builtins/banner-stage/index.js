const fs = require("fs");
const path = require("path");
import png from "./empty.png";
import("./b.js").then(res => {
	// xxxx
});

const prefix = 'PROCESS_ASSETS_STAGE_'

it("should add banner and footer in order", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");

	const banners = [
		`ADDITIONS`, // -100(default)
		`ADDITIONAL`, // -2000
		`OPTIMIZE`, // 100
	];

	// Each banner only occurs once
	expect(
		banners
			.map((banner) => mainFile.split(`/*! ${prefix}${banner} */`).length)
			.every((length) => length === 2)
	);

	// Banners are placed in order
	expect(
		banners
			.map((banner) => [mainFile.indexOf(`/*! ${prefix}${banner} */`), banner])
			.sort(([a], [b]) => a - b)
			.map(([, stage]) => stage)
	).toStrictEqual([
		`OPTIMIZE`, // 100
		`ADDITIONS`, // -100(default)
		`ADDITIONAL`, // -2000
	]);

	const footers = [
		`DERIVED`, // -200
		`REPORT`, // 5000
		`PRE_PROCESS`, // -1000
	];

	// Each footer only occurs once
	expect(
		footers
			.map((footer) => mainFile.split(`/*! ${prefix}${footer} */`).length)
			.every((length) => length === 2)
	);

	// Footers are placed in order
	expect(
		footers
			.map((footer) => [mainFile.indexOf(`/*! ${prefix}${footer} */`), footer])
			.sort(([a], [b]) => a - b)
			.map(([index, stage]) => stage)
	).toStrictEqual([
		`PRE_PROCESS`, // -1000
		`DERIVED`, // -200
		`REPORT`, // 5000
	]);
})

it("should place source map before REPORT", () => {
	const mainFile = fs.readFileSync(__filename, "utf-8");
	expect(mainFile.endsWith("//# sourceMappingURL=main.js.map")).toBeFalsy()
	expect(mainFile.endsWith(`/*! ${prefix}REPORT */`)).toBeTruthy()
})

it("should keep source map", () => {
	expect(fs.existsSync(path.resolve(__dirname, "main.js.map"))).toBe(true);
});

it("should not inject placeholder to asset", () => {
	const pngContent = fs.readFileSync(
		path.resolve(__dirname, "./empty.png"),
		"utf-8"
	);
	expect(pngContent.startsWith(`/*! ${prefix}OPTIMIZE */`)).toBeFalsy();
});
