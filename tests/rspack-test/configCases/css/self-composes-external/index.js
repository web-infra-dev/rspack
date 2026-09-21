import * as styles from "./styles.module.css";

it("should preserve runtime exports while expanding same-file compositions", () => {
  expect(globalThis.__cssSelfComposesEvaluations).toBe(1);
  expect(styles.externalOnly).toBe("externalOnly js-token-1");
  expect(styles.selfOnly).toBe("selfOnly");
  expect(styles.emptyCycle).toBeUndefined();
  expect(styles.selfFirst).toBe("selfFirst js-token-1");
  expect(styles.externalFirst).toBe("externalFirst js-token-1");
  expect(styles.indirect).toBe("indirect selfFirst js-token-1");
  expect(styles.cycleA).toBe("cycleA cycleB js-other-1 js-token-1");
  expect(styles.cycleB).toBe("cycleB cycleA js-token-1 js-other-1");
  expect(styles.withCss).toBe("withCss externalCss");
});

if (process.env.EXPORT_TYPE === "style") {
  it("should evaluate external CSS modules referenced alongside self-composition", () => {
    const css = [...document.getElementsByTagName("style")]
      .map(style => style.textContent)
      .join("\n");
    expect(css).toContain("color: rebeccapurple");
  });
}
