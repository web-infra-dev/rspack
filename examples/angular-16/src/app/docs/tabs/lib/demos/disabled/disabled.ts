import { Component, ViewChild } from '@angular/core';
import { TabsetComponent } from 'ngx-bootstrap/tabs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tabs-disabled',
  templateUrl: './disabled.html'
})
export class DemoTabsDisabledComponent {
  @ViewChild('staticTabs', { static: false }) staticTabs?: TabsetComponent;

  disableEnable() {
    if (this.staticTabs?.tabs[2]) {
      this.staticTabs.tabs[2].disabled = !this.staticTabs.tabs[2].disabled;
    }
  }
}
