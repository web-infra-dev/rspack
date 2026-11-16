it('routes extracted CSS into a shadow root and keeps the original filename expression for fallback', async () => {
  const host = document.createElement('div');
  host.id = 'host';
  document.body.appendChild(host);
  const shadow = host.attachShadow({ mode: 'open' });
  const urls = [];
  const append = shadow.appendChild.bind(shadow);
  shadow.appendChild = link => {
    urls.push(link.href);
    const result = append(link);
    // jsdom doesn't dispatch load for a stylesheet inside a shadow root.
    queueMicrotask(() => link.dispatchEvent(new Event('load')));
    return result;
  };
  try {
    await import(/* webpackChunkName: "first" */ './first.css');
    __webpack_require__.rbLoadStyleSheet = (href, id) => '/backup/' + id + '/' + href;
    await import(/* webpackChunkName: "second" */ './second.css');
    await import(/* webpackChunkName: "isolated" */ './isolated.css');
    expect(urls.map(url => new URL(url).pathname)).toEqual([
      expect.stringMatching(/\/first\.[a-f0-9]{8}\.css$/),
      expect.stringMatching(/^\/backup\/second\/second\.[a-f0-9]{8}\.css$/),
      expect.stringMatching(/\/isolated\.[a-f0-9]{8}\.css$/),
    ]);
    expect(shadow.querySelectorAll('link')).toHaveLength(3);
    expect(document.head.querySelector('link')).toBeNull();
  } finally {
    host.remove();
  }
});
