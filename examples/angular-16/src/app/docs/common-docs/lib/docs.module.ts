import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { BsDropdownModule } from "ngx-bootstrap/dropdown";
import { TabsModule } from 'ngx-bootstrap/tabs';

/* common */
import { AppFooterComponent } from './common/app-footer/app-footer.component';
import { AddNavComponent } from './common/add-nav/add-nav.component';
import { SidebarComponent } from './common/sidebar/sidebar.component';
import { SearchFilterPipe } from './common/search-input/search-filter.pipe';
import { SearchInputComponent } from './common/search-input/search-input.component';
import { BreadCrumbsComponent } from './common/breadCrumbs/breadCrumbs.component';
import { ComponentsPageComponent } from './common/components-page/components-page.component';
import { ConstructionPageComponent } from './common/construction-page/construction-page.component';
import { ThemesComponent } from './common/themes/themes.component';
/* docs */
import { DemoSectionComponent } from './demo-section/demo-section.component';
import { ContentSection } from './models/content-section.model';
import { ExamplesComponent, ExamplesComponentModule } from './demo-section-components/demo-examples-section/index';
import { ApiSectionsComponent, ApiSectionsComponentModule } from './demo-section-components/demo-api-section/index';
import { DocsSectionComponent } from './docs-section/docs-section.component';
import { NgApiDocClassComponent, NgApiDocComponent, NgApiDocConfigComponent, NgApiDocModule } from './api-docs/index';
import { LandingComponent } from './common/landing/landing.component';
import { DocumentationComponent } from './common/documentation/documentation.component';
import { DiscoverComponent } from './common/discover/discover.component';
import { SchematicsComponent } from './common/schematics/schematics.component';
import { TopMenuComponent } from './common/top-menu/top-menu.component';

/* export */
export {
  NgApiDocModule,
  NgApiDocComponent,
  NgApiDocClassComponent,
  NgApiDocConfigComponent
} from './api-docs/index';
export {
  ExamplesComponent,
  ExamplesComponentModule
} from './demo-section-components/demo-examples-section/index';
export {
  ApiSectionsComponent,
  ApiSectionsComponentModule
} from './demo-section-components/demo-api-section/index';
export { DemoSectionComponent } from './demo-section/demo-section.component';
export { DocsSectionComponent } from './docs-section/docs-section.component';
export { SampleBoxComponent } from './api-docs/sample-box/sample-box.component';
export { ContentSection } from './models/content-section.model';
export { AppFooterComponent } from './common/app-footer/app-footer.component';
export { AddNavComponent } from './common/add-nav/add-nav.component';
export { SidebarComponent } from './common/sidebar/sidebar.component';
export { SearchFilterPipe } from './common/search-input/search-filter.pipe';
export { SearchInputComponent } from './common/search-input/search-input.component';
export { LandingComponent } from './common/landing/landing.component';
export { DocumentationComponent } from './common/documentation/documentation.component';
export { DiscoverComponent } from './common/discover/discover.component';
export { SchematicsComponent } from './common/schematics/schematics.component';
export { StyleManager } from './theme/style-manager';
export { ThemeStorage } from './theme/theme-storage';
export { NgApiDoc } from './api-docs/api-docs.model';
export { TopMenuComponent } from './common/top-menu/top-menu.component';
export { Analytics } from './api-docs/analytics/analytics';
export { DOCS_TOKENS } from './tokens/docs-routes-token';
export { SIDEBAR_ROUTES } from './tokens/docs-sidebar-routes-token';
export { SidebarRoutesStructure } from './models/sidebar-routes.model';
export { ComponentsPageComponent } from './common/components-page/components-page.component';
export { ConstructionPageComponent } from './common/construction-page/construction-page.component';
export { ThemesComponent } from './common/themes/themes.component';
import { RouterModule } from '@angular/router';

@NgModule({
  declarations: [
    DemoSectionComponent,
    SidebarComponent,
    AppFooterComponent,
    SearchFilterPipe,
    AddNavComponent,
    DocsSectionComponent,
    LandingComponent,
    DocumentationComponent,
    DiscoverComponent,
    SchematicsComponent,
    SearchInputComponent,
    TopMenuComponent,
    BreadCrumbsComponent,
    ComponentsPageComponent,
    ConstructionPageComponent,
    ThemesComponent
  ],
  imports: [
    CommonModule,
    NgApiDocModule,
    ExamplesComponentModule,
    ApiSectionsComponentModule,
    TabsModule.forRoot(),
    BsDropdownModule.forRoot(),
    RouterModule
  ],
  exports: [
    SearchFilterPipe,
    SidebarComponent,
    AppFooterComponent,
    AddNavComponent,
    DemoSectionComponent,
    ExamplesComponentModule,
    ApiSectionsComponentModule,
    DocsSectionComponent,
    SearchInputComponent,
    TopMenuComponent
  ]
})
export class DocsModule {}
