export const internalDotDotModules = import.meta.glob('./tmp/../../CASE-TEST/*.JS', {
  eager: true,
  caseSensitive: false,
})
