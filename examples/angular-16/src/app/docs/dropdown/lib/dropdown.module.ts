import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { BsDropdownModule } from 'ngx-bootstrap/dropdown';

import { DocsModule } from '../../common-docs';
import { DropdownSectionComponent } from './dropdown-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { routes } from './demo-dropdown.routes';
/*exports*/
export { DropdownSectionComponent } from './dropdown-section.component';

@NgModule({
    declarations: [
        DropdownSectionComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        BsDropdownModule.forRoot(),
        CommonModule,
        FormsModule,
        DocsModule,
        RouterModule.forChild(routes)
    ],
    exports: [DropdownSectionComponent]
})
export class DemoDropdownModule {}
