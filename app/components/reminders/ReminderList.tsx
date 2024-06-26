import React, { useContext } from "react";
import ReminderItem from "./ReminderItem";
import useTranslation from "../../useTranslation";
import { Loader } from "@mantine/core";
import { LanguageContext } from "../../locales/LanguageContext";
interface ReminderListProps {
  enabled: boolean;
}

const ReminderList: React.FC<ReminderListProps> = ({ enabled }) => {
  const { language } = useContext(LanguageContext);
  const dict = useTranslation(language);

  if (!dict) {
    return <Loader color="cyan" type="bars" />;
  }

  const reminders = [
    { id: 1, key: "drink_water", description: dict.reminders.description.drink_water, time: 30, check: true },
    { id: 2, key: "stretch_hands", description: dict.reminders.description.stretch_hands, time: 15, check: false },
    { id: 3, key: "stretch_legs", description: dict.reminders.description.stretch_legs, time: 45, check: true},
    { id: 4, key: "relax_eyes", description: dict.reminders.description.relax_eyes, time: 20, check: true},
    { id: 5, key: "sit_properly", description: dict.reminders.description.sit_properly, time: 60, check: true },
  ];

  return (
    <>
      {reminders.map((reminder) => (
        <ReminderItem
          key={reminder.id}
          reminder={{
            id: reminder.id,
            label: dict.reminders[reminder.key] as string,
            description: reminder.description,
            time: reminder.time,
            check: reminder.check
          }}
          enabled={enabled}
        />
      ))}
    </>
  );
};

export default ReminderList;