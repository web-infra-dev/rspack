import engines from './engines.cjs';
import reassigned from './reassign.cjs';

it('should preserve CommonJS exports assignments', () => {
  expect(engines).toEqual({ yaml: 1 });
  expect(reassigned).toEqual({ original: {}, reassigned: { value: 3 } });
});

export default engines;
