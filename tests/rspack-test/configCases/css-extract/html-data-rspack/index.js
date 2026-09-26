import "./style.css";
import "./native.css";

it("should mark generated native and extracted links with separate ownership keys", () => {
	const html = require("fs").readFileSync(require("path").join(__dirname, HTML_FILE), "utf-8");
	const links = html.match(/<link[^>]*>/g);
	expect(links).toHaveLength(3);
	expect(links).toContain('<link href="framework.css" rel="stylesheet">');
	expect(links.find(link => link.includes("extract-main.css?"))).toContain('data-rspack="html-css:mini-css-chunk-main"');
	expect(links.find(link => link.includes("native-main.css?"))).toContain('data-rspack="html-css:chunk-main"');
});
