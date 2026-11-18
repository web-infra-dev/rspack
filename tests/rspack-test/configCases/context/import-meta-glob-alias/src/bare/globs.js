// Bare patterns that match no alias stay relative to this file.
export const nestedModules = import.meta.glob("nested/*.js", {
  eager: true,
  import: "default",
});
export const queryModules = import.meta.glob("query*.js", {
  query: "?raw",
  import: "default",
});
export const emptyAliasModules = import.meta.glob("dir/*.js", {
  eager: true,
  import: "default",
});
export const mjsQueryModules = import.meta.glob("*.mjs", {
  query: "?raw",
  import: "default",
});
export const explicitRelativeModules = import.meta.glob("./@/glob-imports/*.js", {
  eager: true,
  import: "default",
});
