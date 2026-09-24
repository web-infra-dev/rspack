import { defineConfig } from '@rspack/cli';
import path from 'node:path';

export default defineConfig({
  externals: [
    async ({ context, getResolve }) => {
      const resolve = getResolve!();
      expect(await resolve(context!, './index.js?foo=1#bar=1')).toBe(
        path.join(import.meta.dirname, './index.js') + '?foo=1#bar=1',
      );
      expect(
        await new Promise((promiseResolve, promiseReject) => {
          resolve(context!, './index.js?foo=1#bar=1', (err, result) => {
            if (err) {
              promiseReject(err);
              return;
            }
            promiseResolve(result);
          });
        }),
      ).toBe(path.join(import.meta.dirname, './index.js') + '?foo=1#bar=1');

      const resolveIgnored = getResolve!({ alias: { ignored: false } });
      expect(await resolveIgnored(context!, 'ignored')).toBeUndefined();
      expect(
        await new Promise((promiseResolve, promiseReject) => {
          resolveIgnored(context!, 'ignored', (err, result) => {
            if (err) {
              promiseReject(err);
              return;
            }
            promiseResolve(result);
          });
        }),
      ).toBe(false);
      return false;
    },
  ],
});
