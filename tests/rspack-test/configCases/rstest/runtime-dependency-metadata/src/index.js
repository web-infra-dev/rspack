import { createRequire } from 'node:module';

const nativeRequire = createRequire(import.meta.url);

export function load(locale) {
  return import(`./providers/${locale}.mjs`);
}

export function loadConcatenated(locale) {
  return require('./providers/' + locale + '.cjs');
}

export function resolveProvider(locale) {
  return require.resolve(`./providers/${locale}.cjs`);
}

export function resolveLiteral() {
  return require.resolve('./providers/en.cjs');
}

export function nestedDirectory(locale, name) {
  return import(`./providers/${locale}/${name}.mjs`);
}

export function loadOpaque(request) {
  return import(request);
}

export function ignoredImport(locale) {
  return import(/* webpackIgnore: true */ `./ignored/${locale}.mjs`);
}

export function ignoredRequire(locale) {
  return require(/* webpackIgnore: true */ `./ignored/${locale}.cjs`);
}

export function ignoredLiteralRequire() {
  return require(/* webpackIgnore: true */ './ignored/literal.cjs');
}

export function ignoredResolve() {
  return require.resolve(/* webpackIgnore: true */ './ignored/value.cjs');
}

export function resolveWithOptions(locale) {
  return require.resolve(`./providers/${locale}.cjs`, { paths: ['/custom'] });
}

export function shadowed(require, locale) {
  return [
    require(`./shadowed/${locale}.cjs`),
    require.resolve(`./shadowed/${locale}.cjs`),
  ];
}

export function nested() {
  return import(require('./nested.cjs').request);
}

export function staticConcatenation() {
  return require('./nested' + '.cjs');
}

export function createdRequire(locale) {
  return nativeRequire(`./created/${locale}.cjs`);
}

export function createdResolve(locale) {
  return nativeRequire.resolve(`./created/${locale}.cjs`);
}
