import { foo } from './foo'

it('should compile', () => {
  class Foo {
    foo(a = foo.msg) {
      return a
    }
  }
  expect(new Foo().foo()).toBe('hello')
})
