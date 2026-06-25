export function load() {
  return import("./lazy").then(mod => mod.value);
}

it("loads async chunk through rspack context", async () => {
  await expect(load()).resolves.toBe(7);
});
