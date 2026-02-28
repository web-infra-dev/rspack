import { foo } from './foo';
import { bar } from './bar';

it('static virtual modules should work', () => {
	expect(foo).toBe('foo');
	expect(bar).toBe('bar');
});
