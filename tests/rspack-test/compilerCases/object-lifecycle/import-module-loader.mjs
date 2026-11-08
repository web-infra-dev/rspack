import assert from "node:assert/strict";
import { forceGC } from "@rspack/test-tools/helper/lifecycle";

function importWithPublicPath(context, request, prefix) {
  const captured = { prefix };
  const publicPath = () => captured.prefix;
  context.trackImportCallback(publicPath, captured);
  return context.importModule(request, { publicPath });
}

export default async function (source) {
  this.cacheable(false);
  const pending = Promise.allSettled([
    importWithPublicPath(this, "./import-module-asset.txt?first", "/first/"),
    importWithPublicPath(this, "./import-module-asset.txt?second", "/second/"),
    importWithPublicPath(this, "./import-module-error.js", "/error/"),
  ]);
  await forceGC(2);
  const [first, second, failed] = await pending;
  assert.equal(first.status, "fulfilled", first.reason?.message);
  assert.equal(first.value, "/first/import-module-asset.txt");
  assert.equal(second.status, "fulfilled", second.reason?.message);
  assert.equal(second.value, "/second/import-module-asset.txt");
  assert.equal(failed.status, "rejected");
  assert.match(failed.reason.message, /expected import failure/);
  return source;
}
