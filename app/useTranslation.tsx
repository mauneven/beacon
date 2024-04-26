import { useState, useEffect } from 'react';

interface TranslationDict {
    reminders: {
      [key: string]: string | {
        drink_water: string;
        stretch_hands: string;
        stretch_legs: string;
        relax_eyes: string;
        sit_properly: string;
      };
      title: string;
      info: string;
      settings: string;
      language: string;
      about_beacon: string;
      min_indicator: string;
      hour_indicator: string;
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

export default function useTranslation(lang: string): TranslationDict | null {
  const [dictionary, setDictionary] = useState<TranslationDict | null>(null);

  useEffect(() => {
    import(`./locales/${lang}.json`)
      .then((module) => {
        setDictionary(module.default);
      })
      .catch(() => {
        setDictionary(null);
      });
  }, [lang]);

  return dictionary;
}