const React = {
	createElement(elm: any) {}
};
function Component<T>() {
  return <div></div>
}
export const App = () => <Component<any>></Component>;
