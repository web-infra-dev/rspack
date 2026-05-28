const rspackSourcesFixture = (file) =>
  file.includes('/crates/rspack_sources/tests/fixtures/') ||
  file.includes('/xtask/benchmark/benches/fixtures/rspack_sources/');

const quote = (file) => JSON.stringify(file);

const runWithNonFixtureFiles = (command) => (files) => {
  const filtered = files.filter((file) => !rspackSourcesFixture(file));
  return filtered.length ? `${command} ${filtered.map(quote).join(' ')}` : [];
};

export default {
  '*.rs': 'rustfmt',
  '*.{ts,tsx,js,mjs,yaml,yml}': runWithNonFixtureFiles(
    'node ./node_modules/prettier/bin/prettier.cjs --write',
  ),
  '*.toml': 'pnpm exec taplo format',
  '*.{ts,tsx,js,cts,cjs,mts,mjs}': runWithNonFixtureFiles('pnpm run lint:js'),
  'website/**/*': () => 'pnpm --dir website run check:spell',
  'package.json': () => 'pnpm run check-dependency-version',
};
