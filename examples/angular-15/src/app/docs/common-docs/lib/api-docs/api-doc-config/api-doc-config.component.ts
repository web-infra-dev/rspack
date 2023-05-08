/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
import { Component, ChangeDetectionStrategy } from '@angular/core';
// import docs from '../../../../api-docs';
import { ClassDesc, NgApiDoc } from '../api-docs.model';
import { Analytics } from '../analytics/analytics';
import { ComponentApi } from '../../models/components-api.model';

const CONFIG_SUFFIX_LENGTH = 'Config'.length;

/**
 * Displays the API docs of a Config service. A Config service for a component Foo is named, by convention,
 * FooConfig, and only has properties, whose name matches with an input of the directive.
 * In order to avoid cluttering the demo pages, the only things displayed by this component
 * is the description of the Config service and the list of its properties, whose documentation and
 * default value is documented in the directive itself.
 */
@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'ng-api-doc-config',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './api-doc-config.component.html'
})
export class NgApiDocConfigComponent {
  apiDocs?: ClassDesc;
  directiveName?: string;
  headerAnchor?: string;
  isShowMethods = false;

  private analytics: Analytics;
  private docs: NgApiDoc;

  constructor(analytics: Analytics, docs: NgApiDoc, content: ComponentApi) {
    this.analytics = analytics;
    this.docs = docs;

    this.headerAnchor = content.anchor;
    if (content?.title) {
      this.apiDocs = this.docs[content.title];
    }
    this.isShowMethods = content.showMethods || this.isShowMethods;
    this.directiveName = content.title?.slice(0, -CONFIG_SUFFIX_LENGTH);
  }

  trackSourceClick(): void {
    this.analytics.trackEvent('Source File View', this.apiDocs?.className);
  }
}
