"use client"
import React, { createContext, useState } from "react";

interface LanguageContextType {
  language: string;
  setLanguage: (lang: string) => void;
}

export const LanguageContext = createContext<LanguageContextType>({
  language: "",
  setLanguage: () => {},
});

export const LanguageProvider: React.FC<{ children: React.ReactNode, lang: string }> = ({ children, lang }) => {
  const [language, setLanguage] = useState<string>(lang);

  const handleLanguageChange = (newLang: string) => {
    setLanguage(newLang);
    if (typeof window !== "undefined") {
      localStorage.setItem("language", newLang);
    }
  };

  return (
    <LanguageContext.Provider value={{ language, setLanguage: handleLanguageChange }}>
      {children}
    </LanguageContext.Provider>
  );
};