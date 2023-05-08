/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
import { Component, ChangeDetectionStrategy } from '@angular/core';
import { ClassDesc, MethodDesc, signature, NgApiDoc } from '../api-docs.model';
import { Analytics } from '../analytics/analytics';
import { ComponentApi } from '../../models/components-api.model';

/**
 * Displays the API docs of a class, which is not a directive.
 *
 * For Config services, use NgbdApiDocsConfig instead.
 */
@Component({
  // eslint-disable-next-line @angular-eslint/component-selector
  selector: 'ng-api-doc-class',
  changeDetection: ChangeDetectionStrategy.OnPush,
  templateUrl: './api-doc-class.component.html'
})
export class NgApiDocClassComponent {
  headerAnchor?: string;
  apiDocs?: ClassDesc;

  private analytics: Analytics;
  private docs: NgApiDoc;

  constructor(analytics: Analytics, docs: NgApiDoc, content: ComponentApi) {
    this.docs = docs;
    this.analytics = analytics;

    this.headerAnchor = content.anchor;
    if (content?.title) {
      this.apiDocs = this.docs[content.title];
    }
  }

  methodSignature(method: MethodDesc): string {
    return signature(method);
  }

  trackSourceClick(): void {
    this.analytics.trackEvent('Source File View', this.apiDocs?.className);
  }
}
