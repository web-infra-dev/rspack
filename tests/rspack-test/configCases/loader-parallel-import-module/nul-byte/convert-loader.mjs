import path from "node:path";

export default async function () {
  const importLoader = path.resolve(import.meta.dirname, 'import-loader.mjs');
  const sourceLoader = path.resolve(import.meta.dirname, 'source-loader.mjs');
  const empty = path.resolve(import.meta.dirname, 'empty.js');
  const request = `${importLoader}!${sourceLoader}?{"content": "#b"}!${empty}`;
  return `
    import value from ${JSON.stringify(request)};
    export default value;`
    ;
}
