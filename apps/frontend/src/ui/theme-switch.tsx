"use client";

import { useState, useEffect } from "react";
import { useTheme } from "next-themes";

import { capitalCase } from "change-case";
import {
  Select,
  ListBox,
  ListBoxItem,
  SelectPopover,
  SelectTrigger,
  SelectValue,
  SelectIndicator,
} from "@heroui/react";

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
    <Select
      value={theme}
      onChange={(key) => setTheme(key?.toString() ?? "light")}
    >
      <SelectTrigger className="max-w-min gap-2" aria-label="Pick a theme">
        <SelectValue />
        <SelectIndicator />
      </SelectTrigger>
      <SelectPopover>
        <ListBox>
          {themes.map((themeId) => (
            <ListBoxItem key={themeId} id={themeId} textValue={themeId}>
              {capitalCase(themeId)}
            </ListBoxItem>
          ))}
        </ListBox>
      </SelectPopover>
    </Select>
  );
};

export default ThemeSwitch;
