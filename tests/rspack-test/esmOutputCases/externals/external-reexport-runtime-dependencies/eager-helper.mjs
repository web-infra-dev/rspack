export function load() {
  return import(/* webpackMode: 'eager' */ './bridge.mjs');
}
globalThis.helperLoaded = true;
