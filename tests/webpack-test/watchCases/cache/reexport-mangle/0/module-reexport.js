export { bar } from "./bar?reexport";
export { default as foo } from "./foo?reexport";
export const baz = "baz";

console.log.bind(console);
