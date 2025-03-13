import { PropsWithChildren } from "react";
import { isDev } from "../../utils/isDev";

export const DevModeOnly: React.FC<PropsWithChildren> = ({ children }) => isDev() ? children : null;
