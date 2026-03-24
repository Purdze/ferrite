import { useState } from "react";
import { Installation } from "./types.ts";

export const useInstallations = () => {
  const [installations, setInstallations] = useState<Installation[]>([]);
  const [editingInstall, setEditingInstall] = useState<Installation | null>(null);
  const [activeInstall, setActiveInstall] = useState<Installation | null>(null);
  const [selectedInstall, setSelectedInstall] = useState<Installation | null>(null);

  return {
    installations,
    setInstallations,
    activeInstall,
    setActiveInstall,
    selectedInstall,
    setSelectedInstall,
    editingInstall,
    setEditingInstall,
  };
};
