import { NgModule } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { CommonModule } from '@angular/common';
import { RouterModule, Routes } from '@angular/router';
import { AccordionModule } from 'ngx-bootstrap/accordion';
import { DocsModule } from '../../common-docs';
import { AccordionSectionComponent } from './accordion-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { routes } from './demo-accordion.routes';
/* export */
export { AccordionSectionComponent } from './accordion-section.component';


@NgModule({
    declarations: [
        AccordionSectionComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        AccordionModule.forRoot(),
        CommonModule,
        FormsModule,
        DocsModule,
        RouterModule.forChild(routes),
    ],
    exports: [AccordionSectionComponent]
})
export class DemoAccordionModule {
  static routes: Routes = routes;
}
