import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonVariant = "default" | "primary" | "danger";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  children: ReactNode;
  variant?: ButtonVariant;
}

export function Button({ children, className = "", variant = "default", ...props }: ButtonProps) {
  const variantClass = variant === "default" ? "" : `pb-button--${variant}`;
  return <button className={`pb-button ${variantClass} ${className}`.trim()} {...props}>{children}</button>;
}
