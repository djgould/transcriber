import { useRouter } from "next/router";

export default function useIsTray() {
  const { pathname, asPath } = useRouter();
  const isTray = asPath.includes("tray");

  return isTray;
}
