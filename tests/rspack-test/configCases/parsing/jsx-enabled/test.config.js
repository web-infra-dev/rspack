const fs = require("fs");
const path = require("path");

module.exports = {
	findBundle: () => [],
	validate(stats, stderr, options) {
		const configs = Array.isArray(options) ? options : [options];
		const outputPath = configs[0].output.path;
		const bundle0 = fs.readFileSync(path.join(outputPath, "bundle0.jsx"), "utf-8");
		expect(bundle0).toMatchFileSnapshotSync(
			path.join(__dirname, "__snapshot__", "bundle0.jsx.txt")
		);

		const bundle1 = fs.readFileSync(path.join(outputPath, "bundle1.jsx"), "utf-8");
		expect(bundle1).toContain("<foo:bar value=");
		expect(bundle1).toContain("<svg:path d=");
		expect(bundle1).toContain("<group-container>");
		expect(bundle1).toContain(
			'<NamespaceComponents.Button label="Namespace button"{...{'
		);
		expect(bundle1).toContain('data-dynamic="registry"data-item="one"/>');
		expect(bundle1).toContain(
			'<text-block dangerouslySetInnerHTML={{__html:"<strong>bold</strong>"}}/>'
		);
		expect(bundle1).toContain(
			'<SectionWithSpread {...{"data-testid":"component-with-spread",role:"region"}}/>'
		);
	}
};
