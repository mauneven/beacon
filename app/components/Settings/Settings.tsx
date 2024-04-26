import { Modal, Select } from "@mantine/core";
import React, { useContext } from "react";
import { LanguageContext } from "../../locales/LanguageContext";
import useTranslation from "@/app/useTranslation";

interface SettingsProps {
  opened: boolean;
  onClose: () => void;
}

function Settings({ opened, onClose }: Readonly<SettingsProps>) {
  const { language, setLanguage } = useContext(LanguageContext);
  const dict = useTranslation(language);

  const changeLanguage = (lang: string | null) => {
    if (lang === null) return;
    setLanguage(lang);
  };

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title={dict?.reminders.settings}
      centered
      overlayProps={{
        backgroundOpacity: 0.55,
        blur: 2,
      }}
    >
      <Select
        label={dict?.reminders.language}
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