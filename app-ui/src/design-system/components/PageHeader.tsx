import type { ReactNode } from "react";

interface PageHeaderProps {
  actions?: ReactNode;
  eyebrow?: string;
  title: string;
}

export function PageHeader({ actions, eyebrow, title }: PageHeaderProps) {
  return (
    <header className="pb-page-header flex items-end justify-between gap-6">
      <div>{eyebrow ? <p className="pb-eyebrow">{eyebrow}</p> : null}<h1 className="pb-display">{title}</h1></div>
      {actions}
    </header>
  );
}
