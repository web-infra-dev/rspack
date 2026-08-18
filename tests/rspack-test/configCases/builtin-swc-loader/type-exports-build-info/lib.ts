import type { ImportedType } from './types';

export interface Foo {
  value: number;
}

export type Bar = string;

export enum Baz {
  A,
}

export default interface DefaultType {
  value: string;
}

export type { ImportedType as ReExportedType } from './types';
export { depValue as renamedValue } from './dep';
export * as namespace from './namespace';
export * from './star';

export type Inline = import('./inline-type').InlineType;

export const value = 1;

export const loadDynamic = () => import('./dynamic');
