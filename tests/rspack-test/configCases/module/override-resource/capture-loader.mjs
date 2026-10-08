import { basename, extname } from 'node:path';

export default function (source) {
  const ext = this._module.resourceResolveData.ext;
  if (ext !== extname(this.resourcePath)) {
    throw new Error(`Unexpected resource extension: ${ext}`);
  }
  return JSON.stringify({
    source,
    filename: basename(this.resourcePath),
    ext,
    query: this.resourceQuery,
    fragment: this.resourceFragment,
  });
}
