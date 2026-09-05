import { value } from './sync';
import { answer } from './async';

it('keeps module graph artifacts available during finishModules', () => {
  expect(value).toBe(42);
  expect(answer).toBe(42);
});
