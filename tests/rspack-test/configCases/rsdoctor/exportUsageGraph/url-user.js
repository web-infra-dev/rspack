export function getAssetUrl() {
  return new URL("./url-asset.wasm", import.meta.url).href;
}
