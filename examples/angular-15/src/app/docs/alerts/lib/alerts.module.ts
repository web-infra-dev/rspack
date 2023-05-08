import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule, Routes } from '@angular/router';
import { AlertModule } from 'ngx-bootstrap/alert';
import { AlertsSectionComponent } from './alerts-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { DocsModule } from '../../common-docs';
import { routes } from './demo-alerts.routes';
/* export */
export { AlertsSectionComponent } from './alerts-section.component';

@NgModule({
    declarations: [
        AlertsSectionComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        AlertModule.forRoot(),
        CommonModule,
        DocsModule,
        RouterModule.forChild(routes)
    ],
    exports: [AlertsSectionComponent]
})
export class DemoAlertsModule {
  static routes: Routes = routes;
}
