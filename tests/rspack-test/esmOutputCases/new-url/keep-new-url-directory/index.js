// Directory references are not assets. `new URL(...)` should be kept as-is so
// it resolves to a directory at runtime, matching Node/browsers.
export const dir = new URL('.', import.meta.url).href;
export const dirSlash = new URL('./', import.meta.url).href;
export const parentDir = new URL('..', import.meta.url).href;
