import React from "react";
import ReminderItem from "./ReminderItem";
import useTranslation from "../../useTranslation";
interface ReminderListProps {
  enabled: boolean;
  lang: string;
}

const ReminderList: React.FC<ReminderListProps> = ({ enabled, lang }) => {
  const dict = useTranslation(lang);

  if (!dict) {
    return <div>Loading translations...</div>;
  }

  const reminders = [
    { id: 1, key: "drink_water", description: dict.reminders.description.drink_water },
    { id: 2, key: "stretch_hands", description: dict.reminders.description.stretch_hands },
    { id: 3, key: "stretch_legs", description: dict.reminders.description.stretch_legs },
    { id: 4, key: "relax_eyes", description: dict.reminders.description.relax_eyes },
    { id: 5, key: "sit_properly", description: dict.reminders.description.sit_properly },
  ];

  return (
    <>
      {reminders.map((reminder) => (
        <ReminderItem
          key={reminder.id}
          reminder={{
            id: reminder.id,
            label: dict.reminders[reminder.key] as string,
            description: reminder.description
          }}
          enabled={enabled}
        />
      ))}
    </>
  );
};

export default ReminderList;