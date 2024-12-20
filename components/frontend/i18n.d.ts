import en from "~/locales/en";

type StringKey<T> = Extract<keyof T, string>;
type GenerateKeyPaths<T, Prefix extends string = ""> = T extends object
    ? {
        [K in StringKey<T>]: T[K] extends object
        ? GenerateKeyPaths<T[K], `${Prefix}${K}.`>
        : `${Prefix}${K}`;
    }[StringKey<T>]
    : Prefix;

type Locale = typeof fr;

declare global {
    type I18nKeys = GenerateKeyPaths<Locale>;
}

declare module "@vue/runtime-core" {
    interface ComponentCustomProperties {
        $t: (key: string, ...params: any[]) => string;
        $d: (date: Date, format: string) => string;
        $i18n: I18n;
    }
}