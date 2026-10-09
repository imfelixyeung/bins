"use client";

import { Button, Input } from "@heroui/react";
import { Check, Clipboard, Cross } from "lucide-react";
import { useEffect, useState } from "react";

const Copyable = ({ text }: { text: string }) => {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState(false);
  const [lastCopied, setLastCopied] = useState<number | null>(null);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setError(false);
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (_error) {
      setCopied(false);
      setError(true);
    }
    setLastCopied(Date.now());
  };

  useEffect(() => {
    if (lastCopied === null) return;

    const timeout = setTimeout(() => {
      setCopied(false);
      setError(false);
    }, 2000);

    return () => clearTimeout(timeout);
  }, [lastCopied]);

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
