const SENSITIVE_NAME =
  /(TOKEN|SECRET|PASSWORD|PASSWD|API[_-]?KEY|PRIVATE[_-]?KEY|CONNECTION[_-]?STRING|CREDENTIAL)/i;

export function isSensitiveName(name: string): boolean {
  return SENSITIVE_NAME.test(name);
}

export function displaySensitiveValue(value: string | undefined, sensitive: boolean): string {
  if (value === undefined) return "—";
  return sensitive ? "••••••••••••" : value;
}
