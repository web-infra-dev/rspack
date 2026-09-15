"use server-entry";

const rspackRsc = import.meta.rspackRsc;

export const RscService = () => rspackRsc.loadCss().length;
