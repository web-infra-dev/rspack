// Lazy (default): each value is a thunk () => Promise<module>
const lazyModules = import.meta.glob('./dir/*.js')
const nestedModules = import.meta.glob('./pages/*/index.js')
const rootModules = import.meta.glob('/context/import-meta-glob/dir/*.js')

it('should return a thunk for each matched file in lazy mode', async () => {
  const keys = Object.keys(lazyModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])

  const foo = await lazyModules['./dir/foo.js']()
  expect(foo.default).toBe('foo')

  const bar = await lazyModules['./dir/bar.js']()
  expect(bar.default).toBe('bar')
})

it('should traverse directory wildcard segments in lazy mode', async () => {
  const keys = Object.keys(nestedModules).sort()
  expect(keys).toEqual(['./pages/bar/index.js', './pages/foo/index.js'])

  const foo = await nestedModules['./pages/foo/index.js']()
  expect(foo.default).toBe('nested foo')

  const bar = await nestedModules['./pages/bar/index.js']()
  expect(bar.default).toBe('nested bar')
})

it('should resolve absolute glob patterns from the project root', async () => {
  const keys = Object.keys(rootModules).sort()
  expect(keys).toEqual([
    '/context/import-meta-glob/dir/bar.js',
    '/context/import-meta-glob/dir/foo.js',
  ])

  const foo = await rootModules['/context/import-meta-glob/dir/foo.js']()
  expect(foo.default).toBe('foo')

  const bar = await rootModules['/context/import-meta-glob/dir/bar.js']()
  expect(bar.default).toBe('bar')
})

// Eager: each value is the module object directly
const eagerModules = import.meta.glob('./dir/*.js', { eager: true })

it('should expose module objects directly in eager mode', () => {
  const keys = Object.keys(eagerModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])
  expect(eagerModules['./dir/foo.js'].default).toBe('foo')
  expect(eagerModules['./dir/bar.js'].default).toBe('bar')
})
