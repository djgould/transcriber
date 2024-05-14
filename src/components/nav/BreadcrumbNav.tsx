import Link from "next/link";
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbSeparator,
  BreadcrumbPage,
} from "../ui/breadcrumb";
import { useRouter } from "next/router";
import React from "react";

function generateBreadcrumbs(pathname: string) {
  if (pathname === "/") {
    return (
      <BreadcrumbItem>
        <BreadcrumbPage>Dashboard</BreadcrumbPage>
      </BreadcrumbItem>
    );
  }

  const pathSegments = pathname.split("/").filter((segment) => segment);

  return pathSegments.map((segment, index, arr) => {
    const href = "/" + arr.slice(0, index + 1).join("/");
    const isLast = index === arr.length - 1;
    return (
      <React.Fragment key={href}>
        <BreadcrumbItem>
          {isLast ? (
            <BreadcrumbPage>{segment}</BreadcrumbPage>
          ) : (
            <BreadcrumbLink asChild>
              <Link href={href}>{segment}</Link>
            </BreadcrumbLink>
          )}
        </BreadcrumbItem>
        {!isLast && <BreadcrumbSeparator />}
      </React.Fragment>
    );
  });
}

export default function BreadcrumbNav() {
  const router = useRouter();
  const breadcrumbs = generateBreadcrumbs(router.pathname);
  return <Breadcrumb className="hidden md:flex">{breadcrumbs}</Breadcrumb>;
}
