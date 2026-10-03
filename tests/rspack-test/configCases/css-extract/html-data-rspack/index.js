import "./style.css";
import "./native.css";

it("should preserve ownership when hooks rewrite and reorder CSS URLs", () => {
	const html = require("fs").readFileSync(require("path").join(__dirname, HTML_FILE), "utf-8");
	const links = html.match(/<link[^>]*>/g);
	expect(links).toHaveLength(3);
	expect(links.filter(link => link.includes('data-rspack="html-css:css"')).every(link => link.includes('https://cdn.example.com/'))).toBe(true);
	expect(links).toContain('<link href="framework.css" rel="stylesheet">');
	expect(links.find(link => link.includes("extract-main.css?"))).toContain('data-rspack="html-css:css"');
	expect(links.find(link => link.includes("native-main.css?"))).toContain('data-rspack="html-css:css"');
});
