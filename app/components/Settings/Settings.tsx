import { Modal, Select } from "@mantine/core";
import React, { useState, useEffect } from "react";

interface SettingsProps {
  opened: boolean;
  onClose: () => void;
  setLang: (lang: string) => void;
  currentLang: string;
}

function Settings({ opened, onClose, setLang, currentLang }: Readonly<SettingsProps>) {
  const [selectedLang, setSelectedLang] = useState(currentLang);

  useEffect(() => {
    const defaultLang = navigator.language.startsWith("es") ? "es" : "en";
    setSelectedLang(localStorage.getItem('language') ?? defaultLang);
  }, []);

  const changeLanguage = (lang: string | null) => {
    if (lang === null) return;
    setSelectedLang(lang);
    setLang(lang);
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title="Settings"
      centered
      overlayProps={{
        backgroundOpacity: 0.55,
        blur: 2,
      }}
    >
      <Select
        label="Language"
        placeholder="Select a language"
        value={selectedLang}
        onChange={changeLanguage}
        data={[
          { value: "en", label: "English" },
          { value: "es", label: "Español" },
          { value: "fr", label: "Français" },
        ]}
      />
    </Modal>
  );
}

export default Settings;