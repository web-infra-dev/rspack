import path from "node:path";

export default function (code, map) {
  const expectedSource = path
    .resolve(import.meta.dirname, 'index.css')
    .replace(/\\/g, '/');

  expect(map).toBeTruthy();
  expect(map.sources).toEqual([expectedSource]);

  this.callback(null, code, map);
};
