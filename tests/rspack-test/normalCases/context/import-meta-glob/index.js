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
const lazyDefaultModules = import.meta.glob('./dir/*.js', { import: 'default' })
const lazyNamedModules = import.meta.glob('./dir/*.js', { import: 'named' })
const lazyQueryModules = import.meta.glob('./query/*.js', {
  query: '?raw',
  import: 'default',
})
const eagerObjectQueryModules = import.meta.glob('./query/*.js', {
  query: {
    foo: 'bar',
    raw: true,
    count: 1,
  },
  eager: true,
  import: 'named',
})
const commentModules = import.meta.glob(
  './dir/*.js'
  // for test: annotation contains ")"
  /*
   * for test: annotation contains ")"
   * */
)
const objectKeyModules = Object.keys(import.meta.glob('./dir/*.js'))
const objectValueModules = Object.values(import.meta.glob('./dir/*.js', { eager: true }))
const negativeFirstModules = import.meta.glob(['!**/bar.js', './dir/*.js'], { eager: true })
const filteredDefaultModules = import.meta.glob(['./dir/*.js', '!**/bar.js'], {
  eager: true,
  import: 'default',
})
const lazyFilteredNamedModules = import.meta.glob(['./dir/*.js', '!**/bar.js'], {
  import: 'named',
})
const quotedModules = import.meta.glob("./quoted/*.js", { eager: true })
const escapeModules = import.meta.glob('./escape/**/glob.js', { eager: true })
const nodeModules = import.meta.glob('./dir/node_modules/**')

globalThis.__importMetaGlobSideEffects = []
import.meta.glob('./side-effect/*.js', { eager: true })

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

it('should expose selected default exports in lazy mode', async () => {
  const keys = Object.keys(lazyDefaultModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])

  await expect(lazyDefaultModules['./dir/foo.js']()).resolves.toBe('foo')
  await expect(lazyDefaultModules['./dir/bar.js']()).resolves.toBe('bar')
})

it('should expose selected named exports in lazy mode', async () => {
  await expect(lazyNamedModules['./dir/foo.js']()).resolves.toBe('foo named')
  await expect(lazyNamedModules['./dir/bar.js']()).resolves.toBe('bar named')
})

it('should apply query strings to lazy glob imports without changing keys', async () => {
  const keys = Object.keys(lazyQueryModules)
  expect(keys).toEqual(['./query/foo.js'])
  await expect(lazyQueryModules['./query/foo.js']()).resolves.toBe('?raw')
  expect(lazyQueryModules['./query/foo.js?raw']).toBeUndefined()
})

it('should apply query objects to eager glob imports', () => {
  const keys = Object.keys(eagerObjectQueryModules)
  expect(keys).toEqual(['./query/foo.js'])
  expect(eagerObjectQueryModules['./query/foo.js']).toBe(
    '?foo=bar&raw=true&count=1',
  )
})

it('should parse glob calls with comments in the argument list', async () => {
  const keys = Object.keys(commentModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])

  const foo = await commentModules['./dir/foo.js']()
  expect(foo.default).toBe('foo')
})

it('should work when glob results are wrapped with Object.keys and Object.values', () => {
  expect(objectKeyModules.sort()).toEqual(['./dir/bar.js', './dir/foo.js'])
  expect(objectValueModules.map(mod => mod.default).sort()).toEqual(['bar', 'foo'])
})

it('should apply negative patterns independent of array order', () => {
  const keys = Object.keys(negativeFirstModules).sort()
  expect(keys).toEqual(['./dir/foo.js'])
  expect(negativeFirstModules['./dir/foo.js'].default).toBe('foo')
})

it('should combine import selection with array exclusions in eager mode', () => {
  expect(filteredDefaultModules).toEqual({
    './dir/foo.js': 'foo',
  })
})

it('should combine import selection with array exclusions in lazy mode', async () => {
  const keys = Object.keys(lazyFilteredNamedModules)
  expect(keys).toEqual(['./dir/foo.js'])
  await expect(lazyFilteredNamedModules['./dir/foo.js']()).resolves.toBe('foo named')
})

it('should handle matched paths containing single quotes', () => {
  expect(Object.keys(quotedModules)).toEqual(["./quoted/quote'.js"])
  expect(quotedModules["./quoted/quote'.js"].default).toBe('single-quote')
})

it('should handle relative glob bases inside directories with glob special characters', () => {
  const actual = Object.fromEntries(
    Object.entries(escapeModules).map(([key, mod]) => [key, mod.relative]),
  )
  expect(actual).toEqual({
    './escape/(parenthesis)/glob.js': {
      './mod/index.js': '(parenthesis)',
    },
    './escape/[brackets]/glob.js': {
      './mod/index.js': '[brackets]',
    },
    './escape/{curlies}/glob.js': {
      './mod/index.js': '{curlies}',
    },
  })
})

it('should include explicit node_modules glob matches', async () => {
  const keys = Object.keys(nodeModules)
  expect(keys).toEqual(['./dir/node_modules/hoge.js'])

  const mod = await nodeModules['./dir/node_modules/hoge.js']()
  expect(mod.default).toBe('hoge')
})

it('should execute side effects for unassigned eager glob calls', () => {
  expect(globalThis.__importMetaGlobSideEffects).toEqual(['one', 'two'])
})

// Eager: each value is the module object directly
const eagerModules = import.meta.glob('./dir/*.js', { eager: true })
const eagerDefaultModules = import.meta.glob('./dir/*.js', {
  eager: true,
  import: 'default',
})
const eagerNamedModules = import.meta.glob('./dir/*.js', {
  eager: true,
  import: 'named',
})

it('should expose module objects directly in eager mode', () => {
  const keys = Object.keys(eagerModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])
  expect(eagerModules['./dir/foo.js'].default).toBe('foo')
  expect(eagerModules['./dir/bar.js'].default).toBe('bar')
})

it('should expose eager CommonJS matches as dynamic import namespace objects', () => {
  expect(eagerCjsModules['./cjs/value.js'].default.answer).toBe(42)
})

it('should expose selected default exports in eager mode', () => {
  expect(eagerDefaultModules['./dir/foo.js']).toBe('foo')
  expect(eagerDefaultModules['./dir/bar.js']).toBe('bar')
})

it('should expose selected named exports in eager mode', () => {
  expect(eagerNamedModules['./dir/foo.js']).toBe('foo named')
  expect(eagerNamedModules['./dir/bar.js']).toBe('bar named')
})
