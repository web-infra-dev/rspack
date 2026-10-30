import { defineConfig } from '@rspack/cli';
import path from 'node:path';
import { NormalModule } from '@rspack/core';

export default defineConfig({
  mode: 'development',
  module: {
    rules: [
      {
        test: /\.png$/,
        type: 'asset',
        parser: {
          dataUrlCondition: (
            source: Buffer,
            { filename, module }: { filename: string; module: NormalModule },
          ) => {
            expect(source).toBeInstanceOf(Buffer);
            expect(filename).toBe(
              path.resolve(import.meta.dirname, '../_images/file.png'),
            );
            expect(module).toBeInstanceOf(NormalModule);
            return true;
          },
        },
      },
      {
        test: /\.jpg$/,
        type: 'asset',
        parser: {
          dataUrlCondition: (
            source: Buffer,
            { filename, module }: { filename: string; module: NormalModule },
          ) => {
            expect(source).toBeInstanceOf(Buffer);
            expect(filename).toBe(
              path.resolve(import.meta.dirname, '../_images/file.jpg'),
            );
            expect(module).toBeInstanceOf(NormalModule);
            return false;
          },
        },
      },
      {
        test: /\.svg$/,
        type: 'asset',
        parser: {
          dataUrlCondition: {
            maxSize: 0,
          },
        },
      },
    ],
  },
});
