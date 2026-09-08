const load = () => import('./lazy');
it('retries failed resources in an array chunk load', async () => {
  const first = load();
  const suffix = FAIL_CSS ? 'css-group-' + CASE_INDEX + '.css' : 'js-group-' + CASE_INDEX + '.js';
  const resource = document.head._children.find(element => (element.src || element.href || '').endsWith(suffix));
  expect(resource).toBeDefined();
  resource.onerror({ type: 'error', target: resource });
  await expect(first).rejects.toThrow();
  const retry = load();
  const concurrent = load();
  expect(retry).not.toBe(concurrent);
  const [a, b] = await Promise.all([retry, concurrent]);
  expect(a.default).toBe('ok');
  expect(a).toBe(b);
  expect((await load()).default).toBe('ok');
});
