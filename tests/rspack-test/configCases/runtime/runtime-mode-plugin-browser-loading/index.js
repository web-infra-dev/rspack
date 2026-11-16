it('retries a failed chunk through the native loader and resolves JS and CSS via plugin filenames', async () => {
  const bridge = __webpack_require__.webExtRt;
  const mockId = './mocked.cjs';
  (typeof __rspack_context === 'undefined' ? __webpack_require__ : __rspack_context).rstest_original_module_factories = {
    [mockId]: __webpack_require__.m[mockId] = module => { module.exports = 'kept mock'; },
  };
  // Force only the first script request to fail. Let the runner fetch the
  // second script and the stylesheet and evaluate their actual chunks.
  const original = document.head.appendChild;
  const scriptTags = [];
  const events = [];
  const onError = event => events.push(event);
  document.defaultView.addEventListener('nuxt:preloadError', onError);
  window.pluginSriEnabled = false;
  document.head.appendChild = function (el) {
    if (el.tagName.toLowerCase() === 'script') {
      scriptTags.push([el.integrity, el.crossOrigin, new URL(el.src).hash]);
      if (scriptTags.length === 1) {
        window.pluginSriEnabled = true;
        queueMicrotask(() => el.onerror({ type: 'error', target: el }));
        return el;
      }
    }
    return original.call(this, el);
  };
  try {
    const { value, mocked } = await import(/* webpackChunkName: "lazy" */ './lazy.js');
    expect(value).toBe(42);
    expect(mocked).toBe('kept mock');
    expect(bridge.failures.map(error => error.name)).toEqual(['ChunkLoadError']);
    expect(events).toHaveLength(1);
    expect(events[0].cancelable).toBe(true);
    expect(events[0].payload).toBe(bridge.failures[0]);
    expect(bridge.jsonp).toEqual(['lazy', 'lazy']);
    expect(bridge.scripts).toEqual(['lazy.js', 'lazy.js?retry=1']);
    expect(scriptTags[0][0]).toBeFalsy();
    expect(scriptTags[0][2]).toBe('#bridge');
    expect(scriptTags[1]).toEqual(['test-integrity', 'anonymous', '#bridge']);
    expect(bridge.styles.map(url => new URL(url).pathname.split('/').pop())).toEqual(['lazy.css']);
    expect(__webpack_require__.gca('lazy')).toBe('lazy.js?retry=1');
    expect(__webpack_require__.miniCssF('lazy')).toBe('lazy.css');
    expect(__webpack_require__.gca('probe')).toBe('plugin-output.js');
    expect(__webpack_require__.miniCssF('probe')).toBe('plugin-output.css');
    expect(__webpack_require__.F.plugin('x')).toBe('prefetch:x');
    expect(__webpack_require__.H.plugin('x')).toBe('preload:x');
    expect(new URL(__webpack_require__.b).protocol).toBe('https:');
    const [manifest, hash] = __webpack_require__.hmrM();
    expect(manifest).toContain(hash);
    expect(hash).toBe(__webpack_hash__);
    await bridge.prewarm(['eager']);
    expect(bridge.scripts.at(-1)).toBe('eager.js');
    __webpack_require__.f.i('manual');
    expect(bridge.eagerCalls).toContain('manual');
    expect((await import(/* webpackChunkName: "eager" */ './eager.js')).default).toBe('prewarmed');
  } finally {
    document.defaultView.removeEventListener('nuxt:preloadError', onError);
    delete window.pluginSriEnabled;
    document.head.appendChild = original;
  }
});
