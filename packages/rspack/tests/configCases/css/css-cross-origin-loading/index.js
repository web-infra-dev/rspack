it("should load css chunk without crossOrigin", async function () {
  import("./chunk.css").catch(() => { });
  expect(document.head._children[0]._type).toBe("link");
  expect(document.head._children[0]._href).toBe("https://test.cases/path/19.bundle0.css");
  expect(document.head._children[0].crossOrigin).toBeUndefined();
});
