import { value } from './src/auto-mock';

rs.mock('./src/auto-mock');

it('should fall back to automock when no manual mock exists', () => {
	expect(value).toBe('auto_mock');
});
