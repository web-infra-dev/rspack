// Generate entry modules from their shared-module indices so the overlap is
// visible in the configuration, without duplicating import-only fixtures.
export function createModules(
  count: number,
  entries: Record<string, number[]>,
) {
  const modules = Object.fromEntries(
    Array.from({ length: count }, (_, i) => [
      `m${i}.js`,
      `export default "module-${i}";\n`,
    ]),
  );
  for (const [name, indices] of Object.entries(entries)) {
    modules[`${name}.js`] = `${indices
      .map((i) => `import m${i} from './m${i}';`)
      .join('\n')}
it('preserves exports in ${name}', () => {
  expect([${indices.map((i) => `m${i}`).join(',')}]).toEqual(${JSON.stringify(
    indices.map((i) => `module-${i}`),
  )});
});`;
  }
  return modules;
}

export const modules = createModules(3, {
  a: [0, 1, 2],
  b: [0, 1, 2],
  c0: [0],
  c1: [1],
  c2: [2],
  pair01: [0, 1],
  pair02: [0, 2],
  pair12: [1, 2],
});
export const sharedSize = [0, 1, 2].reduce(
  (size, i) => size + modules[`m${i}.js`].length,
  0,
);
