import * as styles from "./composes.module.css";
import * as escaped from "./escaped.module.css";
import { existsSync, readFileSync } from "fs";
import { join } from "path";

it("should deduplicate direct compositions before and after local declarations", () => {
  expect(styles.forward).toBe("forward leaf");
  expect(styles.backward).toBe("backward leaf");
  expect(styles.mixed).toBe("mixed later");
  expect(styles.external).toBe("external imported");
  expect(styles.externalAgain).toBe("externalAgain imported");
});

it("should preserve shared leaves when reusing completed expansions", () => {
  expect(styles.sharedLeft).toBe("sharedLeft leaf");
  expect(styles.sharedRight).toBe("sharedRight leaf");
  expect(styles.sharedTop).toBe("sharedTop sharedLeft leaf sharedRight leaf");
});

it("should resolve cycles independently for each export root", () => {
  expect(styles.cycleA).toBe("cycleA cycleB");
  expect(styles.cycleB).toBe("cycleB cycleA");
  expect(styles.afterCycle).toBe("afterCycle cycleA cycleB cycleB cycleA");
});

it("should decode escaped custom property names exactly once", () => {
  const cssExports = { ...escaped };
  expect(cssExports["foo\\bar"]).toBeDefined();
  expect(cssExports["fooºr"]).toBeUndefined();
  if (process.env.EXPORTS_ONLY) {
    expect(existsSync(join(__dirname, `bundle${__STATS_I__}.css`))).toBe(false);
    return;
  }
  let css;
  switch (process.env.EXPORT_TYPE) {
    case "text":
      css = cssExports.default;
      break;
    case "css-style-sheet":
      css = Array.from(
        cssExports.default.cssRules,
        (rule) => rule.cssText,
      ).join("\n");
      break;
    case "style":
      css = Array.from(
        document.getElementsByTagName("style"),
        (style) => style.textContent,
      ).join("\n");
      break;
    default:
      css = readFileSync(join(__dirname, `bundle${__STATS_I__}.css`), "utf-8");
  }
  const declaration = css.match(/(--[^\s:;{}]+)\s*:\s*red/);
  const reference = css.match(/var\(\s*([^)]+)\)/);
  expect(declaration).not.toBeNull();
  expect(reference).not.toBeNull();
  expect(reference[1].trim()).toBe(declaration[1]);
});
