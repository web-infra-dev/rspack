import { createContext as cc } from 'preact';
import * as ns from 'preact';
import df from 'preact';

export function aaa(a, b) {
  cc({});
  ns.createContext();
  df.createContext(b);
  return function bbb(a, b, c) {
    cc({});
    ns.createContext();
    df.createContext(b);
  }
}