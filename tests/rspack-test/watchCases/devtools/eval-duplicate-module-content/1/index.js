import { fn as fnA } from './a';
import { fn as fnB } from './b';

export const step = 1;

it('still gives each module its own sourceURL after a rebuild', function () {
  expect(fnA() + fnB()).toBe(84);

  const bundle = require('fs').readFileSync(__filename, 'utf-8');
  const sourceURLs = [
    ...bundle.matchAll(/sourceURL=(\w+:\/\/[^\\\s"]+)/g),
  ].map(([, url]) => url);

  expect(sourceURLs.filter(url => /\/a\.js/.test(url))).toHaveLength(1);
  expect(sourceURLs.filter(url => /\/b\.js/.test(url))).toHaveLength(1);
  expect(new Set(sourceURLs).size).toBe(sourceURLs.length);
});
