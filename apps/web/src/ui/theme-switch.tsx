"use client";

import { useState, useEffect } from "react";
import { useTheme } from "next-themes";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { capitalCase } from "change-case";

const ThemeSwitch = () => {
  const [mounted, setMounted] = useState(false);
  const { theme, setTheme, themes } = useTheme();

  useEffect(() => {
    setMounted(true);
  }, []);

  if (!mounted) {
    return null;
  }

  return (
    <Select value={theme} onValueChange={setTheme}>
      <SelectTrigger className="max-w-min gap-2" aria-label="Pick a theme">
        <SelectValue placeholder="Theme" />
      </SelectTrigger>
      <SelectContent>
        {themes.map((themeId) => (
          <SelectItem key={themeId} value={themeId}>
            {capitalCase(themeId)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
};

export default ThemeSwitch;
