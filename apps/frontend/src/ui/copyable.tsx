"use client";

import { Button } from "@heroui/react";
import { Input } from "@heroui/react";
import React, { useState } from "react";
import { Clipboard, Cross, Check } from "lucide-react";

const Copyable = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState(false);
  const [lastCopied, setLastCopied] = useState(Date.now());

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setError(false);
    } catch (error) {
      setCopied(false);
      setError(true);
    }
    setLastCopied(Date.now());
  };

  React.useEffect(() => {
    const timeout = setTimeout(() => {
      setCopied(false);
      setError(false);
    }, 2000);

    return () => clearTimeout(timeout);
  }, [copied, lastCopied]);

  return (
    <div className="flex w-full items-center gap-2">
      <Input
        type="email"
        placeholder="Email"
        defaultValue={text}
        readOnly
        className="max-w-lg w-full"
      />
      <Button
        type="button"
        onClick={handleCopy}
        className="flex items-center gap-2"
      >
        {copied ? (
          <>
            <Check size="16" />
            Copied!
          </>
        ) : error ? (
          <>
            <Cross size="16" />
            Error..
          </>
        ) : (
          <>
            <Clipboard size="16" />
            Copy
          </>
        )}
      </Button>
    </div>
  );
};

export default Copyable;
