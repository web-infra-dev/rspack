export function load(name) {
  return import(`./stable/${name}.mjs`);
}
