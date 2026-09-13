import { useEffect, useState } from "react";

export type LayoutMode = "compact" | "medium" | "wide";
export type InputMode = "touch" | "pointer";

export interface LayoutCapabilities {
  input: InputMode;
  layout: LayoutMode;
}

export function classifyLayoutCapabilities(width: number, touch: boolean): LayoutCapabilities {
  const layout = width < 640 ? "compact" : width < 1024 ? "medium" : "wide";
  return { input: touch ? "touch" : "pointer", layout };
}

export function detectLayoutCapabilities(): LayoutCapabilities {
  if (typeof window === "undefined") return { input: "pointer", layout: "wide" };
  return classifyLayoutCapabilities(
    window.innerWidth,
    window.matchMedia("(pointer: coarse), (hover: none)").matches,
  );
}

export function useLayoutCapabilities(): LayoutCapabilities {
  const [capabilities, setCapabilities] = useState(detectLayoutCapabilities);
  useEffect(() => {
    const pointer = window.matchMedia("(pointer: coarse), (hover: none)");
    const update = () => setCapabilities(detectLayoutCapabilities());
    window.addEventListener("resize", update);
    pointer.addEventListener("change", update);
    return () => {
      window.removeEventListener("resize", update);
      pointer.removeEventListener("change", update);
    };
  }, []);
  return capabilities;
}
