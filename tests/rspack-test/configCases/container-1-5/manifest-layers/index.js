const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

const statsPath = path.join(__dirname, "mf-stats.json");
const manifestPath = path.join(__dirname, "mf-manifest.json");
const stats = JSON.parse(fs.readFileSync(statsPath, "utf-8"));
const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf-8"));

it("should render layered local and remote shared React", async () => {
  const { default: App } = await import("./host/App.js");
  expect(App()).toBe(
    "App rendered with React version: [This is react 0.1.2]\n" +
      "and remote component: [ComponentA rendered with React version: [This is react 0.1.2] with layer [This is layered react]]\n" +
      " and local component: [ComponentA with React: This is react 0.1.2 layered with This is layered react]",
  );
});

it("should emit manifest files", () => {
  expect(fs.existsSync(statsPath)).toBe(true);
  expect(fs.existsSync(manifestPath)).toBe(true);
});

it("should include the layered consumed shared entry in stats", () => {
  expect(stats.shared).toHaveLength(1);
  expect(stats.shared[0]).toEqual(
    expect.objectContaining({
      name: "react",
      layer: "react-layer",
      singleton: true,
      assets: expect.objectContaining({
        js: expect.objectContaining({
          sync: expect.any(Array),
          async: expect.any(Array),
        }),
        css: expect.objectContaining({
          sync: expect.any(Array),
          async: expect.any(Array),
        }),
      }),
      usedIn: expect.any(Array),
    }),
  );
  expect(stats.shared[0].usedIn.length).toBeGreaterThan(0);
});

it("should include the layered consumed shared entry in manifest", () => {
  expect(manifest.shared).toHaveLength(1);
  expect(manifest.shared[0]).toEqual(
    expect.objectContaining({
      name: "react",
      layer: "react-layer",
      assets: expect.objectContaining({
        js: expect.objectContaining({
          sync: expect.any(Array),
          async: expect.any(Array),
        }),
        css: expect.objectContaining({
          sync: expect.any(Array),
          async: expect.any(Array),
        }),
      }),
    }),
  );
});

it("should include layered expose information in stats", () => {
  expect(stats.exposes).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        name: "local-component",
        path: "./local-component",
        layer: "react-layer",
      }),
    ]),
  );
});

it("should include layered expose information in manifest", () => {
  expect(manifest.exposes).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        name: "local-component",
        path: "./local-component",
        layer: "react-layer",
      }),
    ]),
  );
});
