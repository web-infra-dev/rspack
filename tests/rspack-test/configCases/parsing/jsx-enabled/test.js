const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it("should keep jsx in output when parser jsx is enabled", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle0.jsx"), "utf-8");
  expect(bundle).toMatchFileSnapshotSync(
    path.join(__SNAPSHOT__, "bundle0.jsx.txt")
  );
});

// TODO: There are some clear mangle errors, including but not limited to illegal component names like `t.App1`.
it ("should keep jsx in output when parser jsx is enabled (with minify)", () => {
  const bundle = fs.readFileSync(path.join(__dirname, "bundle1.jsx"), "utf-8");
  expect(bundle).toContain("<foo:bar value=");
  expect(bundle).toContain("<svg:path d=");
  expect(bundle).toContain("<group-container>");
  expect(bundle).toContain("<NamespaceComponents.Button label=\"Namespace button\"{...{");
  expect(bundle).toContain("<App data-dynamic=\"registry\"data-item=\"one\"/>");
  expect(bundle).toContain("<text-block dangerouslySetInnerHTML={{__html:\"<strong>bold</strong>\"}}/>");
  expect(bundle).toContain("<SectionWithSpread {...{\"data-testid\":\"component-with-spread\",role:\"region\"}}/>");
})
