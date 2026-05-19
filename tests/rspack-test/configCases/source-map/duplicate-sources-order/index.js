import "./style.css";

const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("keeps the extracted css source name and falls back for the loader-chain module", () => {
	const cssMap = JSON.parse(
		fs.readFileSync(path.resolve(__dirname, "bundle0.css.map"), "utf-8")
	);
	const jsMap = JSON.parse(
		fs.readFileSync(path.resolve(__dirname, "bundle0.js.map"), "utf-8")
	);
	const canonicalSources = cssMap.sources.filter(
		source => source === "module://./style.css"
	);
	const fallbackSources = jsMap.sources.filter(source =>
		source.startsWith("fallback://")
	);

	expect(canonicalSources).toHaveLength(1);
	expect(fallbackSources).toHaveLength(1);
	expect(jsMap.sources).not.toContain("module://./style.css");
	expect(cssMap.sources).not.toContain("fallback://./style.css");
	expect(fallbackSources[0]).toContain("cssExtractLoader.js!");
	expect(fallbackSources[0]).toContain("css-loader");
	expect(fallbackSources[0]).toMatch(/style\.css$/);
	expect(fallbackSources[0]).not.toBe("fallback://./style.css");
});
