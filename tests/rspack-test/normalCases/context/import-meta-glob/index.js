// Lazy (default): each value is a thunk () => Promise<module>
const lazyModules = import.meta.glob('./dir/*.js')
const wildcardModules = import.meta.glob('./dir/*')
const nestedModules = import.meta.glob('./pages/*/index.js')
const rootModules = import.meta.glob('/context/import-meta-glob/dir/*.js')
const lazyCjsModules = import.meta.glob('./cjs/*.js')
const eagerCjsModules = import.meta.glob('./cjs/*.js', { eager: true })
const dotfileModules = import.meta.glob('./dot/.*.js')
const filteredModules = import.meta.glob(['./dir/*.js', '!**/bar.js'], { eager: true })
const multiModules = import.meta.glob(['./dir/*.js', './other/*.js'], { eager: true })
const lazyMultiModules = import.meta.glob(['./dir/*.js', './other/*.js'])

it('should return a thunk for each matched file in lazy mode', async () => {
  const keys = Object.keys(lazyModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])

  const foo = await lazyModules['./dir/foo.js']()
  expect(foo.default).toBe('foo')

  const bar = await lazyModules['./dir/bar.js']()
  expect(bar.default).toBe('bar')
})

it('should not expose resolver alternative requests in wildcard mode', () => {
  const keys = Object.keys(wildcardModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])
  expect(keys).not.toContain('./dir/foo')
  expect(keys).not.toContain('./dir/bar')
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

it('should resolve lazy CommonJS matches as dynamic import namespace objects', async () => {
  const cjs = await lazyCjsModules['./cjs/value.js']()
  expect(cjs.default.answer).toBe(42)
})

it('should match explicit dotfile glob patterns', async () => {
  const keys = Object.keys(dotfileModules)
  expect(keys).toEqual(['./dot/.hidden.js'])

  const hidden = await dotfileModules['./dot/.hidden.js']()
  expect(hidden.default).toBe('hidden')
})

it('should support negative patterns in glob arrays', () => {
  const keys = Object.keys(filteredModules).sort()
  expect(keys).toEqual(['./dir/foo.js'])
  expect(filteredModules['./dir/foo.js'].default).toBe('foo')
  expect(filteredModules['./dir/bar.js']).toBeUndefined()
})

it('should support multiple glob patterns in eager mode', () => {
  const keys = Object.keys(multiModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js', './other/baz.js'])
  expect(multiModules['./dir/foo.js'].default).toBe('foo')
  expect(multiModules['./dir/bar.js'].default).toBe('bar')
  expect(multiModules['./other/baz.js'].default).toBe('baz')
})

it('should support multiple glob patterns in lazy mode', async () => {
  const keys = Object.keys(lazyMultiModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js', './other/baz.js'])

  const foo = await lazyMultiModules['./dir/foo.js']()
  const bar = await lazyMultiModules['./dir/bar.js']()
  const baz = await lazyMultiModules['./other/baz.js']()

  expect(foo.default).toBe('foo')
  expect(bar.default).toBe('bar')
  expect(baz.default).toBe('baz')
})

// Eager: each value is the module object directly
const eagerModules = import.meta.glob('./dir/*.js', { eager: true })

it('should expose module objects directly in eager mode', () => {
  const keys = Object.keys(eagerModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])
  expect(eagerModules['./dir/foo.js'].default).toBe('foo')
  expect(eagerModules['./dir/bar.js'].default).toBe('bar')
})

it('should expose eager CommonJS matches as dynamic import namespace objects', () => {
  expect(eagerCjsModules['./cjs/value.js'].default.answer).toBe(42)
})
