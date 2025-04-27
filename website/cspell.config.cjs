const { banWords } = require('cspell-ban-words');

module.exports = {
  $schema:
    'https://raw.githubusercontent.com/streetsidesoftware/cspell/main/cspell.schema.json',
  version: '0.2',
  files: ['**/*.{ts,tsx,js,jsx,md,mdx}'],
  dictionaryDefinitions: [
    {
      name: 'project-words',
      path: './project-words.txt',
      addWords: true,
    },
  ],
  dictionaries: ['project-words'],
  ignorePaths: [
    'node_modules',
    '/project-words.txt',
    'package.json',
    'pnpm-lock.yaml',
    'rspack.mp3',
    'doc_build',
    'docs/ptBR/*', //TODO: portuguese language not pass in pnpm check:spell
    'theme/i18n/ptBR.ts', //TODO: portuguese language not pass in pnpm check:spell
  ],
  flagWords: banWords,
  caseSensitive: true,
  allowCompoundWords: true,
  enableFiletypes: ['mdx'],
  words: ['srcăindexāmoduleācss'],
};
