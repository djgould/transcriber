import { Link, ChevronLeft } from "lucide-react";
import { useRouter } from "next/router";
import { PropsWithChildren } from "react";
import NavBar from "../nav/NavBar";

export function MainLayout({ children }: PropsWithChildren) {
  const { pathname } = useRouter();
  return (
    <>
      <NavBar />
      <div className="flex flex-col sm:gap-4 sm:py-4 sm:pl-14">{children}</div>
    </>
  );
}
