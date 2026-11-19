export const en = () => import(/* webpackChunkName: "en" */ './en.js');
export const fr = () => import(/* webpackChunkName: "fr" */ './fr.js');

const events = __webpack_require__.lynx_ce;
const loadedAtStartup = events.loaded;
events.cachedActions.push('render');

it('installs host chunks and resolves the fallback locale through the active loader', async () => {
  const requests = [];
  const sections = [];
  const nativeRequire = __non_webpack_require__;
  const path = require('node:path');
  globalThis.lynx = {
    async requireModuleAsync(url) {
      requests.push(url);
      return nativeRequire(path.join(__dirname, path.basename(url)));
    },
    fetchBundle(url) {
      requests.push(url);
      return { wait: () => ({ code: 0, url }) };
    },
    loadScript(section, { bundleName }) {
      sections.push([section, bundleName]);
      return { value: section === 'main-thread' ? 1 : 2 };
    },
  };
  try {
    expect((await __webpack_require__.loadTranslations('fr')).value).toBe('bonjour');
    expect((await __webpack_require__.loadTranslations('unknown')).value).toBe('hello');
    expect(__webpack_require__.readExternal('main-thread').value).toBe(1);
    expect(__webpack_require__.readExternal('background').value).toBe(2);
    expect(requests).toEqual([
      'https://example.test/fr.cjs',
      'https://example.test/en.cjs',
      'https://example.test/external.bundle',
      'https://example.test/external.bundle',
    ]);
    expect(sections).toEqual([
      ['main-thread', 'https://example.test/external.bundle'],
      ['background', 'https://example.test/external.bundle'],
    ]);
    expect(loadedAtStartup).toBe(false);
    expect(events.loaded).toBe(true);
    expect(events.cachedActions).toEqual([]);
  } finally {
    delete globalThis.lynx;
    delete globalThis.processEvalResultByHost;
  }
});
