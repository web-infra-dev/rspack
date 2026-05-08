// Lazy (default): each value is a thunk () => Promise<module>
const lazyModules = import.meta.glob('./dir/*.js')

it('should return a thunk for each matched file in lazy mode', async () => {
  const keys = Object.keys(lazyModules).sort()
  expect(keys).toEqual(['./dir/bar.js', './dir/foo.js'])

  const foo = await lazyModules['./dir/foo.js']()
  expect(foo.default).toBe('foo')

  const bar = await lazyModules['./dir/bar.js']()
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
