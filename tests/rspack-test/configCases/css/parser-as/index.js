import bareBlock from "./bare-block.css";
import defaultStylesheet from "./default-stylesheet.css";
import explicitStylesheet from "./explicit-stylesheet.css";
import noTrailingSemi from "./no-trailing-semi.css";
import noUrl from "./no-url.css";
import imageSetCss from "./image-set.css";
import quotedUnquotedUrls from "./quoted-unquoted-urls.css";
import customPropsAndStrings from "./custom-props-and-strings.css";
import emptyCss from "./empty.css";
import strayBraceSemi from "./stray-brace-semi.css";
import strayBraceSemiSemi from "./stray-brace-semi-semi.css";
import strayBraceNoSemi from "./stray-brace-no-semi.css";
import unclosedFunc from "./unclosed-func.css";
import unclosedString from "./unclosed-string.css";
import "./link-block.css";
import globalBlock from "./global-block.css";
import nestingAndVars, { active as activeClass, brand as brandVar } from "./nesting-and-vars.css";
import keyframesEof, { spin as spinName } from "./keyframes-eof.module.css";
import globalThenLocal from "./global-then-local.module.css";
import pureBlock, { btn as pureBtn } from "./pure-block.module.css";

const fs = require("fs");
const path = require("path");

it("should resolve bare declaration urls when as is block-contents and ignore them for stylesheet", () => {
  expect(bareBlock).toContain("color: red;");
  expect(bareBlock).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(bareBlock).not.toContain("./pixel.png");

  expect(defaultStylesheet).toBe("color: red; background: url(./pixel.png);\n");
  expect(explicitStylesheet).toBe("color: red; background: url(./pixel.png);\n");
});

it("should handle block-contents without trailing semicolon or without urls", () => {
  expect(noTrailingSemi.trim()).toBe("color: green; padding: 10px");
  expect(noUrl.trim()).toBe("color: blue;");
});

it("should resolve image-set and quoted/unquoted url() inside block-contents", () => {
  expect(imageSetCss).toMatch(/image-set\(url\("pixel-[a-f0-9]+\.png"\) 1x\)/);
  expect(imageSetCss).not.toContain("./pixel.png");

  const matches = quotedUnquotedUrls.match(/pixel-[a-f0-9]+\.png/g);
  expect(matches).toHaveLength(2);
  expect(quotedUnquotedUrls).not.toContain("./pixel.png");
});

it("should ignore custom properties and string brace/quote tricks in block-contents", () => {
  expect(customPropsAndStrings).toContain("--quote: '\"';");
  expect(customPropsAndStrings).toContain("--custom: {a:b};");
  expect(customPropsAndStrings).toContain("--wrapper-shaped: a{color:red};");
  expect(customPropsAndStrings).toContain("content: '}';");
  expect(customPropsAndStrings).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(customPropsAndStrings).not.toContain("./pixel.png");
});

it("should handle empty css in block-contents mode", () => {
  expect(emptyCss).toBe("");
});

it("should match webpack stray right curly brace recovery in block-contents mode", () => {
  // `; }` stops scanning declarations after the stray brace
  expect(strayBraceSemi).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(strayBraceSemi).not.toContain("./pixel.png");
  expect(strayBraceSemi).toContain("url(./other.png)");

  // `; };` recovers at `;` so subsequent declaration resolves
  expect(strayBraceSemiSemi).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(strayBraceSemiSemi).toMatch(/url\("other-[a-f0-9]+\.png"\)/);
  expect(strayBraceSemiSemi).not.toContain("./pixel.png");
  expect(strayBraceSemiSemi).not.toContain("./other.png");

  // ` }` without semicolon ignores stray brace in block item recovery
  expect(strayBraceNoSemi).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(strayBraceNoSemi).toMatch(/url\("other-[a-f0-9]+\.png"\)/);
  expect(strayBraceNoSemi).not.toContain("./pixel.png");
  expect(strayBraceNoSemi).not.toContain("./other.png");
});

it("should tolerate unclosed function call and string tokens in block-contents mode", () => {
  expect(unclosedFunc).toContain("top: anchor-size(--a1 width");
  expect(unclosedString).toContain("content: 'a");
});

it("should support default link exportType and css/global in block-contents mode", () => {
  const bundleCss = fs.readFileSync(path.resolve(__dirname, "bundle0.css"), "utf-8");
  expect(bundleCss).toContain("color: rebeccapurple;");
  expect(bundleCss).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(bundleCss).not.toContain("url(./pixel.png)");

  expect(globalBlock).toContain("color: teal;");
  expect(globalBlock).toContain(".keep-global");
  expect(globalBlock).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(globalBlock).not.toContain("./pixel.png");
});

it("should support nesting, CSS variables, keyframes at EOF, :global/:local, and pure mode", () => {
  expect(activeClass).toBeTruthy();
  expect(brandVar).toBeTruthy();
  expect(nestingAndVars).toMatch(
    new RegExp(`&\\.${activeClass}\\s*\\{[\\s\\S]*url\\("pixel-[a-f0-9]+\\.png"\\)`),
  );
  expect(nestingAndVars).toMatch(/@media \(min-width: 1px\)\s*\{[\s\S]*url\("pixel-[a-f0-9]+\.png"\)/);
  expect(nestingAndVars).not.toContain("./pixel.png");
  expect(nestingAndVars).toContain(`${brandVar}: red;`);
  expect(nestingAndVars).toContain(`color: var(${brandVar})`);

  expect(spinName).toBeTruthy();
  expect(keyframesEof).toContain(`@keyframes ${spinName}`);
  expect(keyframesEof).toContain(`animation: ${spinName} 1s`);

  expect(globalThenLocal).toContain("--g: 1px;");
  expect(globalThenLocal).not.toContain("--local-var: 1px;");
  expect(globalThenLocal).toMatch(/--[^:]*local-var: 1px;/);
  expect(globalThenLocal).toMatch(/color: var\(--[^)]*local-var\);/);

  expect(pureBtn).toBeTruthy();
  expect(pureBlock).toMatch(/url\("pixel-[a-f0-9]+\.png"\),\s*url\("other-[a-f0-9]+\.png"\)/);
  expect(pureBlock).toContain(`.${pureBtn}`);
});
