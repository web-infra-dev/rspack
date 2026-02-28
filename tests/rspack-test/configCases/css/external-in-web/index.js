import * as style from "./style.css";

it("should import an external CSS inside CSS", async () => {
	const links = Array.from(Object.values(window["__LINK_SHEET__"]));
	const css = links.join("\n");

	expect(css).toContain("color: green");
	expect(css).toContain("padding: 10px");
});

// import * as style1 from "http://test.com/import.css";

it("should work with an external URL", () => {
	const url = new URL("https://test.cases/path/url-external.css", import.meta.url);

	expect(url.toString().endsWith("url-external.css")).toBe(true);
});

it("should import an external css dynamically", async () => {
	const x = await import("./dynamic.css");
	expect(Object.keys(x)).toEqual([]);

	const links = Array.from(Object.values(window["__LINK_SHEET__"]));
	const css = links.join("\n");

	expect(css).toContain("color: red");
	expect(css).toContain("background: url(//example.com/image.png) url(https://example.com/image.png)");
	expect(css).toContain("background-image: url(http://example.com/image.png)");
});
