import * as styles from './style.css';
import shared from './shared.svg';

it('should retain the JavaScript export for assets also imported from JavaScript', () => {
  expect(styles).toBeDefined();
  expect(shared).toMatch(/^(data:image\/svg\+xml|shared\.svg)/);
});
