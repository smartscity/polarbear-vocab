import type { ReactNode } from "react";

interface SettingsSectionProps {
  children: ReactNode;
  description: string;
  label: string;
}

export function SettingsSection({ children, description, label }: SettingsSectionProps) {
  return (
    <section className="pb-settings-section">
      <div className="pb-settings-section__copy"><strong>{label}</strong><span>{description}</span></div>
      {children}
    </section>
  );
}
