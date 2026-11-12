import { usedStyles } from 'components';

it('should keep the used CSS module', () => {
  expect(Boolean(usedStyles.used)).toBe(true);
});
