import { Route, Switch } from 'react-router-dom';

import DialogPage from './pages/dialog-page';
import IndexPage from './pages/index-page';
import React from 'react';
import RoutingPage from './pages/routing-page';
import SveltePage from './pages/svelte-page';
import UiLibraryPage from './pages/ui-library-page';

const Routes = () => (
  <Switch>
    <Route path="/" exact={true}>
      <IndexPage />
    </Route>
    <Route path="/dialog" component={DialogPage} />
    <Route path="/ui-library" component={UiLibraryPage} />
    <Route path="/routing" component={RoutingPage} />
    <Route path="/svelte" component={SveltePage} />
  </Switch>
);

export default Routes;
