import { value } from './parent';
it('retains all shared issuer information', () => { expect(value()).toBe(42); });
