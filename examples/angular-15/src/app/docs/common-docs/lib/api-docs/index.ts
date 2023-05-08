/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
// todo: add configuration of base url for source code
import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { Analytics } from './analytics/analytics';
import { NgApiDocComponent } from './api-doc/api-doc.component';
import { NgApiDocClassComponent } from './api-doc-class/api-doc-class.component';
import { NgApiDocConfigComponent } from './api-doc-config/api-doc-config.component';
import { SampleBoxComponent } from './sample-box/sample-box.component';
import { TabsModule } from 'ngx-bootstrap/tabs';
export { NgApiDoc } from './api-docs.model';
export { Analytics } from './analytics/analytics';
export { NgApiDocComponent } from './api-doc/api-doc.component';
export { NgApiDocClassComponent } from './api-doc-class/api-doc-class.component';
export { NgApiDocConfigComponent } from './api-doc-config/api-doc-config.component';
export { SampleBoxComponent } from './sample-box/sample-box.component';

@NgModule({
  imports: [CommonModule, TabsModule],
  declarations: [
    NgApiDocComponent,
    NgApiDocClassComponent,
    NgApiDocConfigComponent,
    SampleBoxComponent
  ],
  exports: [
    NgApiDocComponent,
    NgApiDocClassComponent,
    NgApiDocConfigComponent,
    SampleBoxComponent
  ]
})
export class NgApiDocModule {}
