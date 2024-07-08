declare module '*.module.scss' {
  const classes: { [key: string]: string };
  export default classes;
}

declare module '*.png' {
  const value: string;
  export default value;
}
