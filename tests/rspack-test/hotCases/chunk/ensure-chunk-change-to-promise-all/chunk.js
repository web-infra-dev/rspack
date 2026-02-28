export function test(count) {
  return import("./file").then(({ React, Vue }) => count === 0 ? React : Vue)
}
