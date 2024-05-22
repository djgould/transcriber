import { Link, ChevronLeft } from "lucide-react";
import { useRouter } from "next/router";
import { PropsWithChildren } from "react";

export function TrayLayout({ children }: PropsWithChildren) {
  const { pathname } = useRouter();
  return (
    <div className="flex flex-col sm:gap-4 sm:py-4 sm:pl-14">
      <header className="sticky top-0 z-30 flex h-14 items-center justify-between gap-4 border-b bg-background px-4 sm:static sm:h-auto sm:border-0 sm:bg-transparent sm:px-6">
        {pathname !== "/tray" && (
          <Link href={"/tray"}>
            <ChevronLeft className="h-5 w-5" />
          </Link>
        )}
        <div className="flex-grow"></div>
        <p className="absolute left-1/2 transform -translate-x-1/2">Platy</p>
      </header>
      {children}
    </div>
  );
}
