import type { ReactNode } from 'react';
import { Icon } from '../icons';

const CALLOUT_ICONS = {
  info: 'info',
  tip: 'lightbulb',
  warning: 'triangle-alert',
  danger: 'octagon-alert',
} as const;

export type CalloutType = keyof typeof CALLOUT_ICONS;

const CALLOUT_LABELS: Record<CalloutType, string> = {
  info: 'Note',
  tip: 'Tip',
  warning: 'Warning',
  danger: 'Danger',
};

export function Callout({
  type = 'info',
  title,
  children,
}: {
  type?: CalloutType;
  title?: string;
  children?: ReactNode;
}) {
  const resolved: CalloutType = type in CALLOUT_ICONS ? type : 'info';

  return (
    <aside className="zellij-callout" data-type={resolved}>
      <span className="zellij-callout-icon" aria-hidden="true">
        <Icon name={CALLOUT_ICONS[resolved]} context={`<Callout type="${resolved}">`} size={18} />
      </span>
      <div className="zellij-callout-body">
        <p className="zellij-callout-title">{title ?? CALLOUT_LABELS[resolved]}</p>
        {children}
      </div>
    </aside>
  );
}
