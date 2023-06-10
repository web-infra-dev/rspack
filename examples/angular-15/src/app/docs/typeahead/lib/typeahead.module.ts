import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { TypeaheadModule } from 'ngx-bootstrap/typeahead';

import { DocsModule } from '../../common-docs';
import { TypeaheadSectionComponent } from './typeahead-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { routes } from './demo-typeahead.routes';
/*exports*/
export { TypeaheadSectionComponent } from './typeahead-section.component';

@NgModule({
    declarations: [
        TypeaheadSectionComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        CommonModule,
        FormsModule,
        ReactiveFormsModule,
        DocsModule,
        TypeaheadModule.forRoot(),
        RouterModule.forChild(routes)
    ],
    exports: [TypeaheadSectionComponent]
})
export class DemoTypeaheadModule {}
