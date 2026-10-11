"use client";

import { RouterProvider } from "@heroui/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useRouter } from "next/navigation";
import { ThemeProvider } from "next-themes";
import { NuqsAdapter } from "nuqs/adapters/next/app";
import type React from "react";

const client = new QueryClient();

const Providers = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  const router = useRouter();

  return (
    <QueryClientProvider client={client}>
      <NuqsAdapter>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange={false}
        >
          <RouterProvider navigate={(href, opts) => router.push(href, opts)}>
            {children}
          </RouterProvider>
        </ThemeProvider>
      </NuqsAdapter>
    </QueryClientProvider>
  );
};

export default Providers;
