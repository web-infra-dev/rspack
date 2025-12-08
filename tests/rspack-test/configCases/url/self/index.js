it("should not parse new URL(import.meta.url) to generate a dynamic chunk", () => {
  const path = __non_webpack_require__("path");
  const url = new URL(import.meta.url);
  // On Windows, URL pathname has a leading slash like '/D:/path/to/file'
  // We need to remove it for proper path comparison
  const pathname = process.platform === "win32" && /^\/[a-zA-Z]:/.test(url.pathname)
    ? url.pathname.slice(1)
    : url.pathname;
  const dir = path.dirname(pathname);
  expect(path.normalize(dir)).toEqual(path.normalize(__TEST_SOURCE_PATH__));
})