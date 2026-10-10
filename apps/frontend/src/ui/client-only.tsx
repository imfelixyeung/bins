"use client";

import dynamic from "next/dynamic";
import type React from "react";

const ClientOnly = ({ children }: { children: React.ReactNode }) => children;

export default dynamic(async () => ClientOnly, { ssr: false });
