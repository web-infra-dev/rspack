require("./index.css");
const fs = require("fs");
const path = require("path");

it("should rewrite the css url()", function () {
	const css = fs.readFileSync(path.resolve(__dirname, "main.css"), "utf-8");
	const a = /a: url\("(.*)"\);/.exec(css)[1];
	expect(a.startsWith("./")).toBe(false);
	expect(a.includes("./logo.png")).toBe(false);
	expect(a.endsWith(".png")).toBe(true);
	expect(a === "b3523cb75fe70add.png").toBe(true);
	const b = /b: url\((.*)\);/.exec(css)[1];
	expect(b).toBe(
		'"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAwAAAAOCAYAAAAbvf3sAAAACXBIWXMAABYlAAAWJQFJUiTwAAAAAXNSR0IArs4c6QAAAARnQU1BAACxjwv8YQUAAAEWSURBVHgBjZFRSsNAEIb/3fQAOUK9gfpe6YIovjU3UE+gJ2g8gTcw3sC+itUFrW+BeAN7ATFSKAjbHWcSC+mSlszLDrvfzvzzj0LHmNpZBtBQd4Gf7fuYgHPPueoCe1DK1UsHd7Dzw+P0daQj/SC5h09OzdGktw221vZXUBlVMN0ILPd6O9yzBBXX8CBdv6kWOGa4YLivQJNjM0ia73oTLsBwJjCB5gu4i7CgbsIey5ThEcOfKziTGFOGHypJeZ7jZ/Gbst4x2/fN9h2eGTNvgk92VrDEWLNmfJXLpIYrRy5b4Fs+9v2/pL0oUncI7FuHLI6PK5lJZGoe8qXNvrry27DesoRLpLPmNkTw9yEcxPWJMR+S/AFbfpAZqxwUNQAAAABJRU5ErkJggg=="'
	);
	const c = /c: url\((.*)\);/.exec(css)[1];
	expect(c).toBe("#ccc");
	const d = /d: url\((.*)\);/.exec(css)[1];
	expect(d).toBe("https://rspack.dev/tests/~img.png");
});
