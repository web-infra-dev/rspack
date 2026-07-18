import { value } from './shared';

it('should keep the shared module in the entry chunk', () => {
	expect(value).toBe(42);
});
