import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { RatingModule } from 'ngx-bootstrap/rating';

import { DocsModule } from '../../common-docs';
import { RatingSectionComponent } from './rating-section.component';
import { DEMO_COMPONENTS } from './demos/index';
import { routes } from './demo-rating.routes';
/*exports*/
export { RatingSectionComponent } from './rating-section.component';

@NgModule({
    declarations: [
        RatingSectionComponent,
        ...DEMO_COMPONENTS
    ],
    imports: [
        CommonModule,
        FormsModule,
        DocsModule,
        RatingModule.forRoot(),
        RouterModule.forChild(routes)
    ],
    exports: [RatingSectionComponent]
})
export class DemoRatingModule {}
