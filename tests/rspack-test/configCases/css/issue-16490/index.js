import "./style.module.css";

const fs = require("fs");
const path = require("path");
const css = () => fs.readFileSync(path.join(__dirname, "bundle0.css"), "utf-8");

it("should scope custom counter styles with the position keyword in either order", () => {
	const output = css();
	expect(output).toContain("@counter-style thumbs-scoped {");
	expect(output).toContain("list-style: thumbs-scoped inside;");
	expect(output).toContain("list-style: inside thumbs-scoped;");
	expect(output).toContain("list-style: thumbs-scoped outside;");
	expect(output).toContain("list-style: outside thumbs-scoped;");
	expect(output).toContain("list-style-type: thumbs-scoped;");
	expect(output).toContain("@counter-style inside-scoped {");
	expect(output).toContain("list-style-type: inside-scoped;");
	expect(output).toContain("list-style: outside inside-scoped;");
	expect(output).toContain("list-style: inside;");
});

it("should scope implicit grid lines using the generated area name", () => {
	const output = css();
	expect(output).toContain('"sd-scoped sd-scoped sd-scoped main-scoped');
	expect(output).toContain("grid-area: main-scoped;");
	expect(output).toContain("grid-column: main-scoped-start / main-scoped-end;");
	expect(output).toContain("grid-row: hd-scoped-start / ft-scoped-end;");
	expect(output).toContain(
		"grid-template-columns: repeat(2, [main-scoped-start] 1fr) [main-scoped-end other-end];"
	);
	expect(output).toContain("grid-column: main-scoped-start / other-end;");
	const escapedRule = output.match(/\.escaped-item-scoped\s*\{[^}]*\}/)[0];
	expect(escapedRule).toContain("grid-column: main-scoped-start / main-scoped-end;");
});
