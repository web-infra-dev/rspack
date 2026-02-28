function cb(v) {
  return v
}

const mod = await import('./async.js')
const {value} = cb(mod);

it('should have correct value', () => {
  expect(value()).toBe(42);
})
export {}