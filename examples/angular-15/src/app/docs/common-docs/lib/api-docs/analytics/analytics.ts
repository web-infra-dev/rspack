/**
 * @author ng-team
 * @copyright ng-bootstrap
 */
import { Injectable } from '@angular/core';
import { NavigationEnd, Router } from '@angular/router';
import { Location } from '@angular/common';
import { filter } from 'rxjs/operators';


declare const ga: any;

/**
 * Simple Google Analytics service. Note that all its methods don't do anything
 * unless the app is deployed on ng-bootstrap.github.io. This avoids sending
 * events and page views during development.
 */
@Injectable({providedIn: 'root'})
export class Analytics {
  private enabled: boolean;
  private location: Location;
  private router: Router;

  constructor(location: Location, router: Router) {
    this.location = location;
    this.router = router;
    this.enabled = typeof window != 'undefined' && window.location.href.indexOf('bootstrap') >= 0;
  }

  /**
   * Intended to be called only once. Subscribes to router events and sends a
   * page view after each ended navigation event.
   */
  trackPageViews(): void {
    if (!this.enabled) {
      return;
    }
    this.router.events
      .pipe(
        filter((event: any) => event instanceof NavigationEnd)
      )
      .subscribe(() => {
      if (typeof ga !== 'undefined') {
        ga('send', { hitType: 'pageview', page: this.location.path() });
      }
    });
  }

  /**
   * Sends an event.
   */
  trackEvent(action: string, category?: string): void {
    if (!this.enabled) {
      return;
    }

    if (!category) {
      return;
    }

    if (typeof ga !== 'undefined') {
      ga('send', {
        hitType: 'event',
        eventCategory: category,
        eventAction: action
      });
    }
  }
}
