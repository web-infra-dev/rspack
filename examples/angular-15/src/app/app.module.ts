import { HttpClientModule } from '@angular/common/http';
import { NgModule } from '@angular/core';
import { RouterModule } from '@angular/router';
import { BsDropdownModule } from 'ngx-bootstrap/dropdown';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { ngdoc } from '../ng-api-doc';
import { AppComponent } from './app.component';
import { DOCS_TOKENS, DocsModule, NgApiDoc, SIDEBAR_ROUTES, SidebarRoutesStructure } from './docs/common-docs';
import { routes } from './app.routing';

@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserAnimationsModule,
    DocsModule,
    HttpClientModule,
    RouterModule.forRoot(routes, { useHash: true }),
    BsDropdownModule.forRoot()
  ],
  providers: [
    { provide: NgApiDoc, useValue: ngdoc },
    { provide: DOCS_TOKENS, useValue: routes },
    { provide: SIDEBAR_ROUTES, useValue: SidebarRoutesStructure }],
  bootstrap: [AppComponent]
})
export class AppModule {
}
