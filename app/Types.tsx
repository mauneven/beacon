interface Reminder {
    id: number;
    key: string;  // Clave para buscar la traducción
  }
  
  interface ReminderListProps {
    enabled: boolean;
    lang: string;  // Idioma actual
  }
  
  interface Dictionary {
    reminders: {
      title: string;
      drink_water: string;
      stretch_hands: string;
      stretch_legs: string;
      relax_eyes: string;
      sit_properly: string;
      description: {
        drink_water: string;
        stretch_hands: string;
        stretch_legs: string;
        relax_eyes: string;
        sit_properly: string;
      };
    };
  }
  