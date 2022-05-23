declare module '*.svg' {
  const FC: (props: any) => JSX.Element;
  export default FC;
}
declare module '*.svg?raw' {
  const url: string;
  export default url;
}
