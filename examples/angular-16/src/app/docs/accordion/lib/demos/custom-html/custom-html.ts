import { Component } from '@angular/core';
import { getBsVer, IBsVersion } from 'ngx-bootstrap/utils';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-accordion-custom-html',
  templateUrl: './custom-html.html'
})
export class DemoAccordionCustomHTMLComponent {
  get _getBsVer(): IBsVersion {
    return getBsVer();
  }
}
