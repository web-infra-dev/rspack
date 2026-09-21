import "./style.css";

it("should resolve module-prefixed URLs through package imports", () => {
  const link = document.getElementsByTagName("link")[0];
  const css = getLinkSheet(link);
  expect(css.match(/url\("icon\.svg"\)/g)).toHaveLength(5);
  expect(css).toContain('url("#internal")');
  expect(css).toContain('url("\\23 internal")');
});
