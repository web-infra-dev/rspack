import * as entry from "./entry.css";

it("should emit self-composed CSS imports once with default local identifiers", () => {
  let css;
  if (process.env.EXPORT_TYPE === "link") {
    css = [...document.getElementsByTagName("link")]
      .map(link => link.sheet.css)
      .join("\n");
  } else if (process.env.EXPORT_TYPE === "text") {
    css = entry.default;
  } else if (process.env.EXPORT_TYPE === "css-style-sheet") {
    css = entry.default._cssText;
  } else if (process.env.EXPORT_TYPE === "style") {
    css = [...document.getElementsByTagName("style")]
      .map(style => style.textContent)
      .join("\n");
  }
  // Each directory contains webpack's css-loader/composes-circular.module.css.
  // Keep their requests distinct so an unconditional duplicate cannot hide
  // behind another entry import when preserving media/layer conditions.
  for (const name of ["local-name", "other-name"]) {
    const selectors = css.match(new RegExp(`\\.[^\\s{}]+-${name}\\s*\\{`, "g"));
    expect(selectors).toHaveLength(3);
    expect(new Set(selectors).size).toBe(3);
  }
  expect(css).toContain("@media screen");
  expect(css).toContain("@layer theme");
});

it("should resolve each same-file composition to its existing CSS import module", () => {
  const modules = [];
  const collect = items => {
    for (const module of items) {
      modules.push(module);
      if (module.modules) collect(module.modules);
    }
  };
  collect(__STATS__.children[__STATS_I__].modules);
  for (const name of ["self", "conditional", "layered"]) {
    expect(modules.filter(module => module.name === `./${name}/composes-circular.module.css`
      || module.name === `./${name}/composes-circular.module.css (in styles)`)).toHaveLength(1);
  }
});
