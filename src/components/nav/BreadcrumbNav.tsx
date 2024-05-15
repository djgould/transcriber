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

const breadcrumbConfig: { [key: string]: string } = {
  "": "Dashboard",
  conversations: "Conversations",
  "conversations/new": "New Conversation",
};

function generateBreadcrumbs(pathname: string) {
  if (pathname === "/") {
    return (
      <BreadcrumbItem>
        <BreadcrumbPage>Dashboard</BreadcrumbPage>
      </BreadcrumbItem>
    );
  }

  const pathSegments = pathname.split("/").filter((segment) => segment);

  return (
    <BreadcrumbList>
      {pathSegments.map((segment, index, arr) => {
        const href = "/" + arr.slice(0, index + 1).join("/");
        const isLast = index === arr.length - 1;
        const label =
          breadcrumbConfig[arr.slice(0, index + 1).join("/")] || segment;

        return (
          <React.Fragment key={href}>
            <BreadcrumbItem>
              {isLast ? (
                <BreadcrumbPage>{label}</BreadcrumbPage>
              ) : (
                <BreadcrumbLink asChild>
                  <Link href={href}>{label}</Link>
                </BreadcrumbLink>
              )}
            </BreadcrumbItem>
            {!isLast && <BreadcrumbSeparator />}
          </React.Fragment>
        );
      })}
    </BreadcrumbList>
  );
}

export default function BreadcrumbNav() {
  const router = useRouter();
  const breadcrumbs = generateBreadcrumbs(router.pathname);
  return <Breadcrumb className="hidden md:flex">{breadcrumbs}</Breadcrumb>;
}
