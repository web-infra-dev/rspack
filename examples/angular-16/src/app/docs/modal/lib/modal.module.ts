import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { ModalModule } from 'ngx-bootstrap/modal';

import { DocsModule } from '../../common-docs';
import { ModalSectionComponent } from './modal-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { routes } from './demo-modal.routes';
import { ModalContentComponent } from './demos/service-component/service-component';
import { ModalContentWithInterceptorComponent } from './demos/service-interceptor/service-interceptor';
import { PopoverModule } from 'ngx-bootstrap/popover';
import { TooltipModule } from 'ngx-bootstrap/tooltip';
/*exports*/
export { ModalSectionComponent } from './modal-section.component';


@NgModule({
    declarations: [
        ModalSectionComponent,
        ModalContentComponent,
        ModalContentWithInterceptorComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        ModalModule.forRoot(),
        PopoverModule.forRoot(),
        TooltipModule.forRoot(),
        CommonModule,
        FormsModule,
        DocsModule,
        RouterModule.forChild(routes)
    ],
    exports: [ModalSectionComponent]
})
export class DemoModalModule {}
