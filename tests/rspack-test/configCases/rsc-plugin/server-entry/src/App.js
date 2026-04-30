"use server-entry";

import "./App.css";
import { getChildCssNodes } from "./Child";

const rspackRsc = import.meta.rspackRsc;

export const getCssNodes = () => rspackRsc.loadCss();

export const getInheritedCssNodes = getChildCssNodes;

export const App = async () => {
    return (
        <h1>RSC App</h1>
    );
};
