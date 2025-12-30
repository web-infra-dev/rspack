// 挂载 React 组件的函数
function mountReactComponent() {
  Promise.all([import('react'), import('react-dom/client')]).then(
    ([React, ReactDOM]) => {
      // 创建 React 组件
      const App = () => {
        return React.createElement(
          'div',
          {
            style: {
              padding: '20px',
              border: '1px solid #ccc',
              borderRadius: '8px',
              margin: '10px 0',
            },
          },
          [
            React.createElement(
              'h2',
              { key: 'title' },
              'SharedReact Component',
            ),
          ],
        );
      };

      // 获取挂载点并渲染组件
      const root = ReactDOM.createRoot(document.getElementById('root'));
      root.render(React.createElement(App));
    },
  );
}

mountReactComponent();
