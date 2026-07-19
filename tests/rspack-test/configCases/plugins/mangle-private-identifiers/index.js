class Example {
  constructor() {
    this._longPrivateValue = 40;
    this._reservedPrivateValue = 1;
    this._reflectedPrivateValue = 2;
  }

  _longPrivateMethod() {
    return this._longPrivateValue + this._reservedPrivateValue;
  }
}

const $a = 3;
const example = new Example();

it('should preserve private identifier behavior across chunks', async () => {
  const { readPrivateValue } = await import('./read');
  expect(example._longPrivateMethod() + $a).toBe(44);
  expect(readPrivateValue(example)).toBe(40);
  expect(example['_reflectedPrivateValue']).toBe(2);
});
