"use server-entry";

import "./App.css";
import { Client } from './Client';

const rspackRsc = import.meta.rspackRsc;

export const getCssNodes = () => rspackRsc.loadCss();

export const App = () => {
  return (
    <>
      <h1>RSC CSS link props</h1>
      <Client />
    </>
  );
};
