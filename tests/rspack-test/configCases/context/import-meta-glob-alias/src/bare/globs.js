// Bare patterns that match no alias stay relative to this file.
export const nestedModules = import.meta.glob("nested/*.js", {
  eager: true,
  import: "default",
});
export const queryModules = import.meta.glob("query*.js", {
  query: "?raw",
  import: "default",
});
