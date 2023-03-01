import styled from 'styled-components';
import { css } from 'styled-components';
import React from 'react';
import ReactDOM from 'react-dom';


const Button = styled.button`
  background: transparent;
  border-radius: 3px;
  border: 2px solid palevioletred;
  color: palevioletred;
  margin: 0 1em;
  padding: 0.25em 1em;

  ${(props:{primary?: boolean}) =>
    props.primary &&
    css`
      background: palevioletred;
      color: white;
    `};
`
const Container = styled.div`
  text-align: center;
`
const App = () => {
  return (<Container>
    <Button>Normal Button</Button>
    <Button primary>Primary Button</Button>
  </Container>);

}
ReactDOM.render(<App />, document.getElementById('root'));