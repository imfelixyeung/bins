"use client";

import React from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { NuqsAdapter } from "nuqs/adapters/next/app";

const queryClient = new QueryClient();

const Providers = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <NuqsAdapter>
      <ThemeProvider
        attribute="class"
        defaultTheme="system"
        enableSystem
        disableTransitionOnChange={false}
      >
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </ThemeProvider>
    </NuqsAdapter>
  );
};

export default Providers;
