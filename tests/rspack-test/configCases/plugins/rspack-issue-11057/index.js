const fs = require("fs");

function shouldNotDeleteVar() {
  const unicodeString = 'eéééééééée';
  console.log(unicodeString);
}

shouldNotDeleteVar();

// more detail see: https://github.com/web-infra-dev/rspack/issues/11057
it('should minify unicode characters correctly when swc specify ecma version', () => {
  const content = fs.readFileSync(__filename, "utf-8");
  const r = content.match(/[\"\']e.+e[\"\']/)?.[0];

  expect(r).toBeDefined();
  expect(r.length).toBe(12);
});