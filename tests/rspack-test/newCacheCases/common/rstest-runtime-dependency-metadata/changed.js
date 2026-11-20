export const value = 0;
export function load(name) {
  return import(`./changed/${name}.mjs`);
}
---
export const value = 1;
export function load() {
  return 'removed';
}
