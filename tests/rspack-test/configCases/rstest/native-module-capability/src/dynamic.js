export async function price() {
  const { createRequire: factory } = await import("node:module");
  return factory(import.meta.url)("./provider.cjs").price;
}
