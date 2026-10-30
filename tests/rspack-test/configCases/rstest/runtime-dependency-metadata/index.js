const fs = require("node:fs");
const path = require("node:path");

const readResult = (variant) =>
  JSON.parse(fs.readFileSync(path.join(__dirname, `${variant}.json`), "utf8"));

for (const variant of ["global", "rule", "concatenated"]) {
  it(`collects native runtime request metadata with ${variant} parser settings`, () => {
    const { metadata, modules, concatenated } = readResult(variant);
    expect(metadata.version).toBe(1);
    // A tagged call can be observed through more than one semantic hook.
    // Selection consumes the union of request descriptors.
    const uniqueCalls = (calls) =>
      Array.from(new Set(calls.map((call) => JSON.stringify(call))));
    expect(uniqueCalls(metadata.calls)).toEqual(
      uniqueCalls([
        {
          kind: "import",
          origin: "source",
          parts: ["./providers/", null, ".mjs"],
        },
        {
          kind: "require",
          origin: "entry",
          parts: ["./providers/", null, ".cjs"],
        },
        {
          kind: "resolve",
          origin: "source",
          parts: ["./providers/", null, ".cjs"],
        },
        { kind: "resolve", origin: "source", parts: ["./providers/en.cjs"] },
        {
          kind: "import",
          origin: "source",
          parts: ["./providers/", null, "/", null, ".mjs"],
        },
        {
          kind: "import",
          origin: "source",
          parts: [null],
          unsupported: "opaque-request",
        },
        {
          kind: "import",
          origin: "entry",
          parts: ["./ignored/", null, ".mjs"],
          unsupported: "ignored-call",
        },
        {
          kind: "require",
          origin: "entry",
          parts: ["./ignored/", null, ".cjs"],
          unsupported: "ignored-call",
        },
        {
          kind: "require",
          origin: "entry",
          parts: ["./ignored/literal.cjs"],
          unsupported: "ignored-call",
        },
        {
          kind: "resolve",
          origin: "entry",
          parts: ["./ignored/value.cjs"],
          unsupported: "ignored-call",
        },
        {
          kind: "resolve",
          origin: "source",
          parts: ["./providers/", null, ".cjs"],
          unsupported: "resolve-options",
        },
        {
          kind: "import",
          origin: "source",
          parts: [null],
          unsupported: "opaque-request",
        },
        {
          kind: "require",
          origin: "entry",
          parts: ["./created/", null, ".cjs"],
          unsupported: "create-require-origin",
        },
        {
          kind: "resolve",
          origin: "entry",
          parts: ["./created/", null, ".cjs"],
          unsupported: "create-require-origin",
        },
        {
          kind: "require",
          origin: "source",
          parts: [],
          unsupported: "native-module-capability",
        },
      ]),
    );
    // Native providers, including the invalid unused one, never enter the
    // compilation. The nested literal require still follows normal parsing.
    expect(modules.map((name) => path.basename(name)).sort()).toEqual(
      variant === "concatenated"
        ? ["entry.js", "index.js", "nested.cjs"]
        : ["index.js", "nested.cjs"],
    );
    expect(concatenated).toBe(variant === "concatenated");

    const output = fs.readFileSync(
      path.join(__dirname, `${variant}.js`),
      "utf8",
    );
    expect(output).toContain("__rstest_import__");
    expect(output).toContain("__rstest_resolve__");
    expect(output).toContain("require('./providers/' + locale + '.cjs')");
    expect(output).toContain("`./ignored/${locale}.mjs`");
    expect(output).toContain("`./shadowed/${locale}.cjs`");
    expect(output).toContain(
      "module.exports = { request: './providers/en.mjs' }",
    );
  });
}

it("uses the same metadata under global and rule-level parser overrides", () => {
  expect(readResult("global").metadata).toEqual(readResult("rule").metadata);
});

it("leaves requests handled by a rule-level require parser in the ordinary graph", () => {
  const { metadata, modules } = readResult("bundled-require");
  expect(metadata).toEqual({ version: 1, calls: [] });
  expect(modules.some((name) => path.basename(name) === "en.cjs")).toBe(true);
});
