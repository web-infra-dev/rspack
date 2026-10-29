import value from "./mod.js";
import "./inline.css";

it("loads an initial chunk's first stylesheet with the require chunk loader", async () => {
	expect(value).toBe("no-css");
	expect(document.querySelectorAll("style")).toHaveLength(1);
	expect(typeof __webpack_require__.hmrC.css).toBe("function");
	expect(document.querySelectorAll('link[rel="stylesheet"]')).toHaveLength(0);

	await NEXT_HMR();

	expect(value).toBe("with-css");
	const links = Array.from(document.querySelectorAll('link[rel="stylesheet"]'));
	expect(links.map(link => link.getAttribute("href"))).toEqual([
		expect.stringContaining("main.css?hmr=")
	]);
	expect(getLinkSheet(links[0])).toContain("color: red");
});

module.hot.accept("./mod.js");
