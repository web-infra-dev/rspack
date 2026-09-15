import { App } from '../App';

export const value = App();

it('should build the main RSC entry', () => {
  expect(value).toBe('main');
});
