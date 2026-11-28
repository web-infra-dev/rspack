export default function (code) {
  // Runs directly after builtin:lightningcss-loader.
  if (this.resourcePath.endsWith("index.css")) {
    // Lightning CSS does not expand custom media in `@import` conditions, so
    // the definition must not be removed while the import still references it.
    expect(code).toContain('@import "./imported.css" (--wide);');
    expect(code).toContain("@custom-media --wide (width >= 800px);");
  }

  return code;
}
