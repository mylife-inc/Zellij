'use client';

import { useEffect } from 'react';

// The tracker owns document-wide history listeners and permits one init only.
// Keep React remounts/Strict Mode from counting the initial page twice.
let initialized = false;

export function Analytics({ domain }: { domain: string }) {
  useEffect(() => {
    let active = true;
    void import('@plausible-analytics/tracker').then(({ init }) => {
      if (!active || initialized) return;
      init({ domain });
      initialized = true;
    }).catch(() => {
      // A blocked analytics chunk must never prevent reading or navigation.
    });
    return () => { active = false; };
  }, [domain]);
  return null;
}
