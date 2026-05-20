import "./styles-1.link.css";
import "./styles-2.link.css";
import "./styles-3.link.css";
import text from "./styles-4.text.css";
import textImport from "./import.text.css";
import styleSheet from "./styles-5.css-style-sheet.css";
import "./styles-6.style.css";
import textInherited from "./styles-7.text.css";
import textInheritedDirect from "./inherit-charset.text.css";

const expectSingleTopCharset = css => {
	expect(css.match(/@charset/g)).toEqual(["@charset"]);
	expect(css.startsWith('@charset "UTF-8";\n')).toBe(true);
};

it("should handle `@charset` at-rule", () => {
	const links = document.getElementsByTagName("link");
	const css1 = [];

	// Skip first because import it by default
	for (const link of [...links].slice(1).filter(link => link.sheet)) {
		css1.push(link.sheet.css);
	}

	if (css1.length > 0) {
		expect(css1.join("\n")).toContain(".class-1");
		expect(css1.join("\n")).toContain(".class-2");
		expect(css1.join("\n")).toContain(".class-3");
	}
	expectSingleTopCharset(text);
	expect(text).toContain(".class-4");
	expectSingleTopCharset(textImport);
	expect(textImport).toContain(".import");
	if (styleSheet._cssText) {
		expectSingleTopCharset(styleSheet._cssText);
		expect(styleSheet._cssText).toContain(".class-5");
	} else {
		expect(styleSheet).toBeInstanceOf(CSSStyleSheet);
	}
	// styles-7 has its own @charset and should keep exactly one directive at byte 0.
	expectSingleTopCharset(textInherited);
	expect(textInheritedDirect).toContain(".inherit-charset");

	const styles = window.document.getElementsByTagName("style");
	const css2 = [];

	for (const style of [...styles]) {
		css2.push(style.textContent);
	}

	expect(css2.join("\n")).toContain(".class-6");
});
