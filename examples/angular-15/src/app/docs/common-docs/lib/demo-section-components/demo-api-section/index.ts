/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
// todo: add configuration of base url for source code
import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ApiSectionsComponent } from './api-sections.component';
import { NgApiDocModule } from '../../api-docs/index';

export { ApiSectionsComponent } from './api-sections.component';

@NgModule({
  declarations: [
    ApiSectionsComponent
  ],
  imports: [
    CommonModule,
    NgApiDocModule
  ],
  exports: [
    ApiSectionsComponent
  ]
})
export class ApiSectionsComponentModule {}
