import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

import { BsDropdownModule } from 'ngx-bootstrap/dropdown';

@NgModule({
  imports: [
    BrowserAnimationsModule,
    BsDropdownModule.forRoot()
  ]
})
export class AppModule(){}

Also should be added web-animations-js polyfill for IE browser (Edge)
