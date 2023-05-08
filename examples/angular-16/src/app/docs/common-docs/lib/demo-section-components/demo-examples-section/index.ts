/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
// todo: add configuration of base url for source code
import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ExamplesComponent } from './examples.component';
import { NgApiDocModule } from '../../api-docs/index';
import { RouterModule } from '@angular/router';
import { TabsModule } from 'ngx-bootstrap/tabs';

export { ExamplesComponent } from './examples.component';

@NgModule({
  declarations: [
    ExamplesComponent
  ],
  imports: [
    CommonModule,
    NgApiDocModule,
    RouterModule,
    TabsModule.forRoot(),
  ],
  exports: [
    ExamplesComponent,
    RouterModule
  ]
})
export class ExamplesComponentModule {}
