import { a } from './a';

if (a !== 2) {
  throw new Error(`mocked value should survive loading the test chunk, got ${a}`);
}
