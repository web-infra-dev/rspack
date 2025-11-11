import { Divider, ThemeProvider, Typography } from '@mui/material';

import Dialog from './Dialog';
import { HashRouter } from 'react-router-dom';
import React from 'react';
import Tabs from './Tabs';
import { theme } from './theme';

const Page = React.lazy(() => import('app_01/Page'));

function App() {
  return (
    <HashRouter>
      <ThemeProvider theme={theme}>
        <React.Suspense fallback={null}>
          <Page title="Material UI App">
            <Typography variant="h6">Dialog Component</Typography>
            <Dialog />
            <Divider style={{ margin: '16px 0' }} />
            <Typography variant="h6">Tabs Component</Typography>
            <Tabs />
          </Page>
        </React.Suspense>
      </ThemeProvider>
    </HashRouter>
  );
}

export default App;
