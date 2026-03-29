import path from 'node:path';

const toProjectRelativePath = (filePath) => {
  const relativePath = path.isAbsolute(filePath)
    ? path.relative(process.cwd(), filePath)
    : filePath;
  return relativePath.replaceAll(path.sep, '/');
};

const runJsLint = (files) => {
  const lintableFiles = files
    .map(toProjectRelativePath)
    .filter((filePath) => filePath.startsWith('packages/rspack/'));

  if (!lintableFiles.length) {
    return [];
  }

  return [
    `pnpm run lint:js -- ${lintableFiles
      .map((filePath) => `"${filePath}"`)
      .join(' ')}`,
  ];
};

export default {
  '*.rs': 'rustfmt',
  '*.{ts,tsx,js,mjs,yaml,yml}':
    'node ./node_modules/prettier/bin/prettier.cjs --write',
  '*.toml': 'pnpm exec taplo format',
  '*.{ts,tsx,js,cts,cjs,mts,mjs}': runJsLint,
  'website/**/*': () => 'pnpm --dir website run check:spell',
  'package.json': () => 'pnpm run check-dependency-version',
};
