import { foo } from './foo';
import { bar } from './bar';

it('two instances of plugin should have no conflicts', () => {
	expect(foo).toBe('foo');
	expect(bar).toBe('bar');
});
