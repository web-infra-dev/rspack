import value from './value.txt';

it('should initialize loader context APIs before invoking loader hooks', () => {
  expect(value).toEqual({ hookData: true, loaderIndex: 0 });
});
