export const baz = async () => {
  const asyncBar = await import('./bar.js')
  return asyncBar
}
