const fs = require('fs');
const path = require('path');

const loadShared = () => import('./module');
module.exports = loadShared;

for (const suffix of ['', '-stats']) {
  it(`should report resolved singleton options in ${suffix || 'manifest'}`, async () => {
    const data = JSON.parse(
      fs.readFileSync(path.join(__dirname, `mf-${CASE_INDEX}${suffix}.json`), 'utf-8'),
    );
    const singletons = Object.fromEntries(
      data.shared.map(item => [item.name, item.singleton]),
    );
    expect(singletons).toMatchObject({
      'ds/Button': false,
      'ds/Table': false,
      'ds/Exact': true,
      'ds/Default': true,
      'renamed/Child': false,
      '@default/lib/Child': true,
    });

    // Overlapping prefixes must follow the actual consumer, even if a provider
    // matched a different prefix. Do not assume a longest-prefix rule here.
    const consumes = Object.values(
      __webpack_require__.consumesLoadingData.moduleIdToConsumeDataMapping,
    );
    expect(consumes).toHaveLength(7);
    expect(data.shared).toHaveLength(7);
    for (const consume of consumes) {
      if (['ds/Default', '@default/lib/Child'].includes(consume.shareKey)) {
        // Preserve the existing manifest default even though the runtime defaults to false.
        expect(consume.singleton).toBe(false);
        expect(singletons[consume.shareKey]).toBe(true);
      } else {
        expect(singletons[consume.shareKey]).toBe(consume.singleton);
      }
    }

    if (!CONSUME_ONLY) {
      const shared = await loadShared();
      expect(shared.Button).toBe('Button');
      expect(shared.Table).toBe('Table');
      expect(shared.Exact).toBe('Exact');
      expect(shared.DefaultExact).toBe('DefaultExact');
      expect(shared.Nested).toBe('Nested');
      expect(shared.Aliased).toBe('Aliased');
      expect(shared.DefaultPrefix).toBe('DefaultPrefix');
    }
  });
}
