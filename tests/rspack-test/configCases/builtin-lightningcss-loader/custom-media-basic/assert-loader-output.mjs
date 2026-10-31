export default function (code) {
  // Runs directly after builtin:lightningcss-loader, before Rspack parses the module.
  expect(code).not.toContain("@custom-media");
  expect(code).not.toContain("(--wide)");
  expect(code).toContain(
    "@media (width >= 800px) {\n  .example {\n    color: red;\n  }\n}",
  );

  return code;
}
