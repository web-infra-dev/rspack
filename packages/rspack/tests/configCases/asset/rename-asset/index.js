const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

it('Should rename chunk asset', async () => {
  await expect(import("./chunk.js")).rejects.toEqual(
    expect.objectContaining({
      code: "ENOENT"
    })
  );
  expect(fs.existsSync(path.join(__dirname, "./chunk.js"), "utf-8")).toBe(false);
  expect(fs.readFileSync(path.join(__dirname, "./renamed.js"), "utf-8")).toContain("chunk value");
});