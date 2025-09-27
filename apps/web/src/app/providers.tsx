"use client";

import React from "react";
import { ThemeProvider } from "next-themes";
import { NuqsAdapter } from "nuqs/adapters/next/app";
import { TRPCProvider } from "@/trpc/client/provider";

const Providers = ({ children }: Readonly<{ children: React.ReactNode }>) => {
  return (
    <TRPCProvider>
      <NuqsAdapter>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange={false}
        >
          {children}
        </ThemeProvider>
      </NuqsAdapter>
    </TRPCProvider>
  );
};

export default Providers;
