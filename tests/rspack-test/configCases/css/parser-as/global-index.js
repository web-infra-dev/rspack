import globalPlain from "./global-plain.css";
import globalAuto from "./global-auto.css";
import globalCssGlobal from "./global-css-global.css";
import globalCssModule, { box as moduleBox } from "./global-css-module.module.css";
import overrideStylesheet from "./override-stylesheet.css";

it("should apply module.parser.*.as globally, inherit css.as into css/auto, and allow overrides", () => {
  expect(globalPlain).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(globalPlain).not.toContain("./pixel.png");

  expect(globalAuto).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(globalAuto).not.toContain("./pixel.png");

  expect(globalCssGlobal).toBe("color: red; background: url(./pixel.png);\n");

  expect(moduleBox).toBeTruthy();
  expect(globalCssModule).toMatch(/url\("pixel-[a-f0-9]+\.png"\)/);
  expect(globalCssModule).toContain(`.${moduleBox}`);
  expect(globalCssModule).not.toContain("./pixel.png");

  expect(overrideStylesheet).toBe("color: red; background: url(./pixel.png);\n");
});
