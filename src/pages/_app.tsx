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
import { ReactElement, ReactNode } from "react";
import { NextPage } from "next";

export type NextPageWithLayout<P = {}, IP = P> = NextPage<P, IP> & {
  getLayout?: (page: ReactElement) => ReactNode;
};

type AppPropsWithLayout = AppProps & {
  Component: NextPageWithLayout;
};

export default function App({ Component, pageProps }: AppPropsWithLayout) {
  const getLayout = Component.getLayout ?? ((page) => page);
  return (
    <Providers>
      <div className="flex min-h-screen w-full flex-col bg-muted/40">
        {getLayout(<Component {...pageProps} />)}
      </div>
      <Toaster />
    </Providers>
  );
}
