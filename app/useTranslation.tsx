
"use client"
import { useState, useEffect } from 'react';

interface TranslationDict {
  reminders: {
    [key: string]: string | {
      drink_water: string;
      stretch_hands: string;
      stretch_legs: string;
      relax_eyes: string;
      sit_properly: string;
    } | Record<string, any>;
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
    about_page: {
      main_title: string;
      about_beacon: string;
      about_beacon_desc_part1: string;
      about_beacon_desc_part2: string;
      about_beacon_desc_part3: string;
      privacy: string;
      privacy_desc: string;
      license: string;
      license_desc: string;
      maintainers: string;
      maintainers_desc: string;
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