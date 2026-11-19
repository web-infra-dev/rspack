it('routes extracted CSS into a shadow root and keeps the original filename expression for fallback', async () => {
  const host = document.createElement('div');
  host.id = 'host';
  document.body.appendChild(host);
  const shadow = host.attachShadow({ mode: 'open' });
  const urls = [];
  const append = shadow.appendChild.bind(shadow);
  shadow.appendChild = link => {
    const pathname = new URL(link.href).pathname;
    urls.push(pathname);
    const result = append(link);
    // jsdom doesn't dispatch load for a stylesheet inside a shadow root.
    const fails = /\/second\.[a-f0-9]{8}\.css$/.test(pathname) && !pathname.startsWith('/backup/');
    queueMicrotask(() => link.dispatchEvent(new Event(fails ? 'error' : 'load')));
    return result;
  };
  try {
    await import(/* webpackChunkName: "first" */ './first.css');
    await import(/* webpackChunkName: "second" */ './second.css');
    await import(/* webpackChunkName: "isolated" */ './isolated.css');
    expect(urls).toEqual([
      expect.stringMatching(/\/first\.[a-f0-9]{8}\.css$/),
      expect.stringMatching(/\/second\.[a-f0-9]{8}\.css$/),
      expect.stringMatching(/^\/backup\/second\/second\.[a-f0-9]{8}\.css$/),
      expect.stringMatching(/\/isolated\.[a-f0-9]{8}\.css$/),
    ]);
    expect(shadow.querySelectorAll('link')).toHaveLength(3);
    expect(document.head.querySelector('link')).toBeNull();
  } finally {
    host.remove();
  }
});
