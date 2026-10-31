const expectExpanded = (code) => {
  expect(code).not.toContain("@custom-media");
  expect(code).not.toContain("(--wide)");
  expect(code).toContain("800px");
};

const expectPreserved = (code) => {
  expect(code).toContain("@custom-media --wide");
  expect(code).toContain("@media (--wide)");
};

export default function (code) {
  // Runs directly after builtin:lightningcss-loader.
  if (this.resourcePath.endsWith("deep.css")) {
    // Preserving parser flags must keep `nonStandard.deepSelectorCombinator` working.
    expectExpanded(code);
    expect(code).toContain(".parent >>> .child");
  } else {
    switch (this.resourceQuery) {
      case "?targets":
        expectExpanded(code);
        break;
      case "?no-draft":
      case "?exclude":
      case "?no-targets":
        expectPreserved(code);
        break;
      default:
        throw new Error(`Unexpected resource query: ${this.resourceQuery}`);
    }
  }

  return code;
}
