import "./styles-1.link.css";
import "./styles-2.link.css";
import "./styles-3.link.css";
import text from "./styles-4.text.css";
import textImport from "./import.text.css";
import styleSheet from "./styles-5.css-style-sheet.css";
import "./styles-6.style.css";
import textInherited from "./styles-7.text.css";
import textInheritedDirect from "./inherit-charset.text.css";

const expectCssWithCharset = (css, expected) => {
	expect(typeof css).toBe("string");
	expect(css.match(/@charset/g)).toEqual(["@charset"]);
	expect(css.startsWith('@charset "UTF-8";\n')).toBe(true);
	for (const item of expected) {
		expect(css).toContain(item);
	}
};

it("should handle `@charset` at-rule", () => {
	const links = document.getElementsByTagName("link");
	const css1 = [];

	// Skip first because import it by default
	for (const link of [...links].slice(1)) {
		if (link.sheet) {
			css1.push(link.sheet.css);
		}
	}

	const linkedCss = css1.find(css => css.includes(".class-3"));
	expect(linkedCss).toBeDefined();
	expectCssWithCharset(linkedCss, [
		'@import url("http://some/path/to/css1.css")',
		".import",
		".class-1",
		".class-2",
		".class-3",
	]);
	expectCssWithCharset(text, [".import-nested", ".import", ".class-4"]);
	expectCssWithCharset(textImport, [".import-nested", ".import"]);
	expectCssWithCharset(styleSheet._cssText, [
		".import-nested",
		".import",
		".class-5",
	]);
	expectCssWithCharset(textInherited, [
		".leaf-with-charset",
		".inherit-charset",
		".class-7",
	]);
	expectCssWithCharset(textInheritedDirect, [
		".leaf-with-charset",
		".inherit-charset",
	]);

	const styles = window.document.getElementsByTagName("style");
	const css2 = [];

	for (const style of [...styles]) {
		css2.push(style.textContent);
	}

	expect(css2.length).toBeGreaterThanOrEqual(4);
	for (const css of css2) {
		expectCssWithCharset(css, []);
	}
	expect(css2.join("\n")).toContain(".class-6");
});
