import type { ReactNode } from "react";

import { useI18n } from "../lib/i18n";
import { useLayoutCapabilities } from "./useLayoutCapabilities";

export type NavScreen = "home" | "datasets" | "listening" | "mistakes" | "settings";

interface AppShellProps {
  appName: string;
  children: ReactNode;
  footer: string;
  onNavigate: (screen: NavScreen) => void;
  screen: NavScreen | "study";
}

const navigation: NavScreen[] = ["home", "datasets", "listening", "mistakes", "settings"];

export function AppShell(props: AppShellProps) {
  const { input, layout } = useLayoutCapabilities();
  const studying = props.screen === "study";
  if (studying) {
    return <div className="pb-app-shell" data-input={input} data-layout={layout}><main className="pb-shell-main">{props.children}</main></div>;
  }
  if (layout === "wide") {
    return (
      <div className="pb-app-shell pb-shell-wide" data-input={input} data-layout={layout}>
        <aside className="pb-rail"><Brand name={props.appName} /><Navigation {...props} /></aside>
        <ShellMain footer={props.footer}>{props.children}</ShellMain>
      </div>
    );
  }
  return (
    <div className="pb-app-shell" data-input={input} data-layout={layout}>
      <ShellMain footer={props.footer}>
        <header className="pb-topbar"><Brand name={props.appName} />{layout === "medium" ? <Navigation {...props} /> : null}</header>
        {props.children}
      </ShellMain>
      {layout === "compact" ? <Navigation {...props} bottom /> : null}
    </div>
  );
}

function ShellMain({ children, footer }: { children: ReactNode; footer: string }) {
  return <main className="pb-shell-main">{children}<footer>{footer}</footer></main>;
}

function Brand({ name }: { name: string }) {
  const { t } = useI18n();
  return <div className="pb-brand"><p className="pb-eyebrow">{t("app.tagline")}</p><strong>{name}</strong></div>;
}

function Navigation(props: AppShellProps & { bottom?: boolean }) {
  const { t } = useI18n();
  return (
    <nav aria-label={t("nav.primary")} className={props.bottom ? "pb-bottom-nav" : "pb-nav"}>
      {navigation.map((screen) => (
        <button data-active={props.screen === screen} key={screen} onClick={() => props.onNavigate(screen)} type="button">{t(`nav.${screen}`)}</button>
      ))}
    </nav>
  );
}
