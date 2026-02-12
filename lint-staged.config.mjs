export default {
  '*.rs': 'rustfmt',
  '*.{ts,tsx,js,mjs,yaml,yml}':
    'node ./node_modules/prettier/bin/prettier.cjs --write',
  '*.toml': 'pnpm exec taplo format',
  '*.{ts,tsx,js,cts,cjs,mts,mjs}': 'pnpm run lint:js',
  'package.json': () => 'pnpm run check-dependency-version',
};
