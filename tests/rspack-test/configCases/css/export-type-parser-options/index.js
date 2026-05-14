import moduleText, { class as moduleTextClass } from "./module-text.css";
import autoText from "./auto-text.css";
import * as moduleTextNoEsm from "./module-text-no-esm.css";
import autoTextNoEsm from "./auto-text-no-esm.css";
import stylesheet from "./stylesheet.css";
import moduleStylesheet, {
  secondary as moduleStylesheetSecondary,
} from "./module-stylesheet.css";
import "./style-for-inject.css";

it("should export CSS text as default when parser exportType is text for css/module", () => {
  expect(typeof moduleText).toBe("string");
  expect(moduleText).toContain("color: red");
  expect(moduleText).toContain("background: white");
  expect(moduleTextClass).toBeTruthy();
});

it("should export CSS text as default when parser exportType is text for css/auto", () => {
  expect(typeof autoText).toBe("string");
  expect(autoText).toContain(".auto-text-class");
  expect(autoText).toContain("color: green");
});

it("should export CSS text when parser exportType is text and esModule is false", () => {
  expect(moduleTextNoEsm["no-esm-text"]).toBeTruthy();
  expect(moduleTextNoEsm.default["no-esm-text"]).toBeTruthy();
  expect(typeof autoTextNoEsm).toBe("string");
  expect(autoTextNoEsm).toContain(".auto-no-esm-text");
  expect(autoTextNoEsm).toContain("color: brown");
});

it("should export CSSStyleSheet when parser exportType is css-style-sheet for css/auto", () => {
  expect(stylesheet).toBeInstanceOf(CSSStyleSheet);
  expect(stylesheet.cssRules.length).toBeGreaterThan(0);

  const rules = Array.from(stylesheet.cssRules);
  const stylesheetRule = rules.find(
    rule => rule.selectorText === ".stylesheet-class",
  );
  expect(stylesheetRule).toBeDefined();
  expect(stylesheetRule.style.color).toBe("purple");
  expect(stylesheetRule.style["font-weight"]).toBe("bold");
});

it("should export CSSStyleSheet when parser exportType is css-style-sheet for css/module", () => {
  expect(typeof moduleStylesheetSecondary).toBe("string");
  expect(moduleStylesheet).toBeInstanceOf(CSSStyleSheet);
  expect(moduleStylesheet.cssRules.length).toBeGreaterThan(0);

  const rules = Array.from(moduleStylesheet.cssRules);
  const moduleRule = rules.find(
    rule => rule.selectorText && rule.selectorText.includes("module-stylesheet"),
  );
  expect(moduleRule).toBeDefined();
  expect(moduleRule.style.color).toBe("orange");
  expect(moduleRule.style.padding).toBe("20px");
});

it("should inject CSS when parser exportType is style", () => {
  const styles = document.getElementsByTagName("style");
  const allStyleText = Array.from(styles)
    .map(style => style.textContent)
    .join("\n");
  expect(allStyleText).toContain("inject-with-style");
  expect(allStyleText).toContain("color: teal");
});
