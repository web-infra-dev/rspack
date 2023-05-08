import { Component, ViewChild } from '@angular/core';
import { TabsetComponent } from 'ngx-bootstrap/tabs';

@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'demo-tabs-manual-selection',
  templateUrl: './manual-selection.html'
})
export class DemoTabsManualSelectionComponent {
  @ViewChild('staticTabs', { static: false }) staticTabs?: TabsetComponent;

  selectTab(tabId: number) {
    if (this.staticTabs?.tabs[tabId]) {
      this.staticTabs.tabs[tabId].active = true;
    }
  }
}
