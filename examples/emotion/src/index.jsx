import ReactDOM from 'react-dom';
import { css, Global, ClassNames } from '@emotion/react'

const App = () => {
  return (
    <div css={{ color: 'hotpink' }}>
      rspack
      <div
        css={css`
        color: green;
      `}
      >loves</div>
      <Global
        styles={{
          body: {
            margin: 0,
            padding: 0
          }
        }}
      />
      <ClassNames>
        {({ css, cx }) => (
          <div
            className={cx(
              'some-class',
              css`
              color: yellow;
            `
            )}
          >emotion</div>
        )}
      </ClassNames>
    </div>
  )
}

ReactDOM.render(<App />, document.getElementById('root'));