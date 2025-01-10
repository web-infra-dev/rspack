import * as json from './data.toml';

it('should use custom parse function', () => {
	expect(json.foo).toBe('bar');
});
