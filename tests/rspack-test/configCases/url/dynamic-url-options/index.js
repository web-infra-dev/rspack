import { getName, getCalls } from './name.js';

it('should control dynamic URL contexts without changing static assets', () => {
  const staticUrl = new URL('./assets/static.txt', import.meta.url);
  expect(staticUrl.pathname).toMatch(
    STATIC_URL_ENABLED ? /bundled\/static\.txt$/ : /assets\/static\.txt$/,
  );

  let name = 'dynamic';
  const dynamicUrl = new URL(`./assets/${name}.txt?query=1`, import.meta.url);
  if (DYNAMIC_URL_ENABLED) {
    expect(dynamicUrl).toMatch(/bundled\/dynamic\.txt\?query=1$/);
  } else {
    expect(dynamicUrl.pathname).toMatch(/assets\/dynamic\.txt$/);
    expect(dynamicUrl.search).toBe('?query=1');
    expect(dynamicUrl instanceof URL).toBe(true);
  }
});

it('should walk dynamic URL arguments and evaluate their side effects once', () => {
  const before = getCalls();
  const url = new URL(`./assets/${getName()}.txt`, import.meta.url);
  expect(getCalls()).toBe(before + 1);
  if (DYNAMIC_URL_ENABLED) {
    expect(url).toMatch(/bundled\/dynamic\.txt$/);
  } else {
    expect(url.pathname).toMatch(/assets\/dynamic\.txt$/);
    expect(url instanceof URL).toBe(true);
  }
});

if (!DYNAMIC_URL_ENABLED) {
  it('should preserve nested static URLs inside a dynamic expression', () => {
    const url = new URL(new URL('./assets/static.txt', import.meta.url).href, import.meta.url);
    expect(url.pathname).toMatch(
      STATIC_URL_ENABLED ? /bundled\/static\.txt$/ : /assets\/static\.txt$/,
    );
  });

  it('should leave wholly dynamic requests to the runtime', () => {
    let request = './not-an-asset.txt';
    const url = new URL(request, import.meta.url);
    expect(url.pathname).toMatch(/dynamic-url-options\/not-an-asset\.txt$/);
  });
}
