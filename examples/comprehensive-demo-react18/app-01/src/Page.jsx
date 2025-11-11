import { AppBar, Toolbar, Typography, Box } from '@mui/material';

import React from 'react';

function Page({ title, children }) {
  return (
    <Box sx={{ flex: 1 }}>
      <AppBar position="relative">
        <Toolbar>
          <Typography variant="h6" noWrap>
            {title}
          </Typography>
        </Toolbar>
      </AppBar>
      <Box sx={{ flexGrow: 1, bgcolor: (theme) => theme.palette.background.default, p: 3 }}>
        {children}
      </Box>
    </Box>
  );
}

export default Page;
