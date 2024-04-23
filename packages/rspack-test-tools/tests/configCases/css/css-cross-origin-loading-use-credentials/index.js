it("should load css chunk with crossOrigin=`use-credentials`", async function () {
  import("./chunk.css").catch(() => { });
  expect(document.head._children[0]._type).toBe("link");
  expect(document.head._children[0]._href).toBe("https://test.cases/path/19.bundle0.css");
  expect(document.head._children[0].crossOrigin).toBe("use-credentials");
});
