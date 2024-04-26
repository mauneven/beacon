import { Modal, Select } from "@mantine/core";
import React, { useContext } from "react";
import { LanguageContext } from "../../locales/LanguageContext";

interface SettingsProps {
  opened: boolean;
  onClose: () => void;
}

function Settings({ opened, onClose }: Readonly<SettingsProps>) {
  const { language, setLanguage } = useContext(LanguageContext);

  const changeLanguage = (lang: string | null) => {
    if (lang === null) return;
    setLanguage(lang);
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
        value={language}
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