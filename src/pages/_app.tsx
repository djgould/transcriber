import Providers from "@/providers";
import type { AppProps } from "next/app";
import Image from "next/image";

import "@/styles/index.css";
import BreadcrumbNav from "@/components/nav/BreadcrumbNav";
import NavBar from "@/components/nav/NavBar";
import { Sheet, SheetTrigger, SheetContent } from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Toaster } from "@/components/ui/toaster";

import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuItem,
} from "@/components/ui/dropdown-menu";
import {
  PanelLeft,
  Package2,
  Home,
  ShoppingCart,
  Package,
  Users2,
  LineChart,
  Search,
  ChevronsLeftRightIcon,
  ChevronLeft,
} from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/router";

export default function App({ Component, pageProps }: AppProps) {
  const { pathname } = useRouter();
  return (
    <Providers>
      <div className="flex min-h-screen w-full flex-col bg-muted/40">
        <NavBar />
        <div className="flex flex-col sm:gap-4 sm:py-4 sm:pl-14">
          <header className="sticky top-0 z-30 flex h-14 items-center justify-between gap-4 border-b bg-background px-4 sm:static sm:h-auto sm:border-0 sm:bg-transparent sm:px-6">
            {pathname !== "/" && (
              <Link href={"/"}>
                <ChevronLeft className="h-5 w-5" />
              </Link>
            )}
            <div className="flex-grow"></div>
            <p className="absolute left-1/2 transform -translate-x-1/2">
              Platy
            </p>
          </header>
          <Component {...pageProps} />
        </div>
      </div>
      <Toaster />
    </Providers>
  );
}
